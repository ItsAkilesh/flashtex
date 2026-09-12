use flashtex_rendering_core::{digest, pdf_export::*, pdf_stream::*};
const FIXTURE: &[u8] = include_bytes!("fixtures/synthetic-pdf-stream.json");
fn stream() -> PdfCommandStream {
    PdfCommandStream::from_mixed_replay(FIXTURE, StreamLimits::default()).unwrap()
}
#[test]
fn actual_pdf_exact_operators_page_geometry_and_provenance() {
    let source = stream();
    let content = source.content_bytes().unwrap();
    let output = export(&[source], ExportLimits::default()).unwrap();
    assert_eq!(
        output.bytes(),
        include_bytes!("fixtures/synthetic-exact-export.pdf")
    );
    assert_eq!(
        output.bytes(),
        export(&[stream()], ExportLimits::default())
            .unwrap()
            .bytes()
    );
    flashtex_pdf::verify::check_structure(output.bytes()).unwrap();
    let file = flashtex_pdf::reader::PdfFile::parse(output.bytes()).unwrap();
    let pages = file.pages().unwrap();
    assert_eq!(pages.len(), 1);
    let decoded = file.page_content(pages[0]).unwrap();
    assert_eq!(decoded, content);
    assert_eq!(
        flashtex_pdf::exact::parse(&decoded).unwrap(),
        flashtex_pdf::exact::parse(&content).unwrap()
    );
    assert!(file.page_fonts(pages[0]).is_empty());
    let evidence: serde_json::Value = serde_json::from_slice(output.evidence_bytes()).unwrap();
    assert_eq!(evidence["pdf_sha256"], digest(output.bytes()));
    assert_eq!(evidence["pages"][0]["content_sha256"], digest(&content));
    assert_eq!(
        evidence["pages"][0]["operator_evidence"]["source_sha256"],
        digest(FIXTURE)
    );
    assert_eq!(
        evidence["pages"][0]["operator_evidence"]["original_fixture"],
        serde_json::from_slice::<serde_json::Value>(FIXTURE).unwrap()
    );
    assert_eq!(evidence["paper"], "white");
    assert_eq!(evidence["preview_theme_applied"], false);
    assert_eq!(evidence["visual_oracle_verified"], false);
    assert_eq!(evidence["content_readback_verified"], true);
    assert!(output.bytes().starts_with(b"%PDF-1.4"));
    assert!(!String::from_utf8_lossy(output.bytes()).contains("/ID"));
    assert!(!String::from_utf8_lossy(output.bytes()).contains("CreationDate"));
}
#[test]
fn atomic_document_limits_and_nonterminating_geometry_refusal() {
    assert!(matches!(
        export(&[], ExportLimits::default()),
        Err(ExportError::Budget)
    ));
    for limits in [
        ExportLimits {
            max_pdf_bytes: 1,
            ..ExportLimits::default()
        },
        ExportLimits {
            max_content_bytes: 1,
            ..ExportLimits::default()
        },
        ExportLimits {
            max_evidence_bytes: 1,
            ..ExportLimits::default()
        },
        ExportLimits {
            max_pages: 1,
            ..ExportLimits::default()
        },
    ] {
        let streams = if limits.max_pages == 1 {
            vec![stream(), stream()]
        } else {
            vec![stream()]
        };
        assert!(matches!(export(&streams, limits), Err(ExportError::Budget)));
    }
    let mut fixture: serde_json::Value = serde_json::from_slice(FIXTURE).unwrap();
    // The first quadratic move now has a nonterminating exact rational operand.
    fixture["primitives"][0]["geometry"]["commands"][0][1][0] = serde_json::json!(["1", "3"]);
    let bytes = serde_json::to_vec(&fixture).unwrap();
    let stream = PdfCommandStream::from_mixed_replay(&bytes, StreamLimits::default()).unwrap();
    assert!(matches!(
        export(&[stream], ExportLimits::default()),
        Err(ExportError::Stream(PdfStreamError::NonTerminatingDecimal))
    ));
}

#[test]
fn empty_original_glyph_retains_provenance_without_invalid_fill() {
    let mut value: serde_json::Value = serde_json::from_slice(FIXTURE).unwrap();
    value["primitives"][0]["geometry"]["commands"] = serde_json::json!([]);
    value["commands"] = 3.into();
    let bytes = serde_json::to_vec(&value).unwrap();
    let stream = PdfCommandStream::from_mixed_replay(&bytes, StreamLimits::default()).unwrap();
    assert_eq!(stream.spans().len(), 3);
    let first = &stream.spans()[0];
    assert!(!stream.operators()[first.start..first.end].contains(&PdfOperator::FillNonZero));
    let exported = export(&[stream], ExportLimits::default()).unwrap();
    let evidence: serde_json::Value = serde_json::from_slice(exported.evidence_bytes()).unwrap();
    assert_eq!(
        evidence["pages"][0]["operator_evidence"]["original_fixture"]["primitives"][0]
            ["original_gid"],
        1
    );
    assert_eq!(
        evidence["pages"][0]["operator_evidence"]["primitive_spans"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
}

#[test]
fn pinned_stix_outline_fixture_exports_identical_real_pdf() {
    let evidence: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/stix-exact-export.evidence.json")).unwrap();
    let page = &evidence["pages"][0]["operator_evidence"];
    let source = serde_json::to_vec(&page["original_fixture"]).unwrap();
    // Source JSON byte formatting is metadata; geometry/GID identities are exact.
    let ticks = page["page_ticks"].as_array().unwrap();
    let stream = PdfCommandStream::from_shaped_replay(
        &source,
        PdfPage {
            width: flashtex_rendering_core::Tick(ticks[0].as_i64().unwrap()),
            height: flashtex_rendering_core::Tick(ticks[1].as_i64().unwrap()),
        },
        flashtex_rendering_core::Paint {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        StreamLimits::default(),
    )
    .unwrap();
    let pdf = export(&[stream], ExportLimits::default()).unwrap();
    assert_eq!(
        pdf.bytes(),
        include_bytes!("fixtures/stix-exact-export.pdf")
    );
    assert_eq!(
        digest(pdf.bytes()),
        "d9df3bf55c2b2dcb126717b9a479965b7a7fd71f39733ff1c52a10d4cd019714"
    );
    let file = flashtex_pdf::reader::PdfFile::parse(pdf.bytes()).unwrap();
    let content = file.page_content(file.pages().unwrap()[0]).unwrap();
    assert_eq!(
        digest(&content),
        "e67b45a267111a19dce157d3492b62ddc54ef357e1916b94e351ce2b8abebac3"
    );
    assert_eq!(flashtex_pdf::exact::parse(&content).unwrap().len(), 245);
    let bytes = serde_json::to_vec(&page["original_fixture"]).unwrap();
    assert!(String::from_utf8(bytes)
        .unwrap()
        .contains("c4864ca6ec071c2d31d0d8309001faa1ee3517fffb53a31a405a697b71f52ca1"));
}
