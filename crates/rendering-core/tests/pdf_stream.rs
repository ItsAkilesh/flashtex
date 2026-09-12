use flashtex_rendering_core::{
    digest,
    outlines::{OutlineCoordinate as Q, OutlinePoint as P},
    pdf_stream::*,
};
use serde_json::{json, Value};
fn q(n: i128, d: u128) -> Q {
    Q::from_fraction(n, d).unwrap()
}
fn point(x: i64, y: i64) -> Value {
    json!([[x.to_string(), "1"], [y.to_string(), "1"]])
}
fn finite_fixture() -> Vec<u8> {
    include_bytes!("fixtures/synthetic-pdf-stream.json").to_vec()
}
#[test]
fn exact_degree_elevation_y_flip_clip_white_paper_and_provenance() {
    let fixture = finite_fixture();
    let stream = PdfCommandStream::from_mixed_replay(&fixture, StreamLimits::default()).unwrap();
    assert_eq!(stream.source_sha256(), digest(&fixture));
    assert_eq!(stream.spans().len(), 3);
    assert_eq!(stream.operators()[1], PdfOperator::FillRgb([q(1, 1); 3]));
    assert_eq!(
        stream
            .operators()
            .iter()
            .find(|o| matches!(o, PdfOperator::Cubic { .. }))
            .unwrap(),
        &PdfOperator::Cubic {
            control1: P {
                x: q(2, 1048576),
                y: q(98, 1048576)
            },
            control2: P {
                x: q(4, 1048576),
                y: q(98, 1048576)
            },
            end: P {
                x: q(6, 1048576),
                y: q(100, 1048576)
            }
        }
    );
    let content = String::from_utf8(stream.content_bytes().unwrap()).unwrap();
    assert!(content.starts_with("q\n1 1 1 rg\n"));
    assert!(content.contains("W\nn\n0 0 0 rg\n"));
    assert!(!content.contains("BT"));
    assert!(!content.contains("Tj"));
    // Independent token-level structure: each primitive isolates graphics state,
    // includes a nonzero clip, and emits valid operator arities.
    let mut depth = 0;
    let mut fills = 0;
    let mut curves = 0;
    for line in content.lines() {
        let parts: Vec<_> = line.split_whitespace().collect();
        let op = *parts.last().unwrap();
        let n = parts.len() - 1;
        let arity = match op {
            "q" => {
                depth += 1;
                0
            }
            "Q" => {
                depth -= 1;
                assert!(depth >= 0);
                0
            }
            "rg" => 3,
            "re" => 4,
            "W" | "n" | "h" => 0,
            "f" => {
                fills += 1;
                0
            }
            "m" | "l" => 2,
            "c" => {
                curves += 1;
                6
            }
            _ => panic!("unknown operator {op}"),
        };
        assert_eq!(n, arity);
        for number in &parts[..n] {
            assert!(number
                .bytes()
                .all(|c| c.is_ascii_digit() || c == b'.' || c == b'-'));
        }
    }
    assert_eq!((depth, fills, curves), (0, 4, 2));
    let evidence: Value = serde_json::from_slice(&stream.evidence_bytes().unwrap()).unwrap();
    assert_eq!(
        evidence["original_fixture"],
        serde_json::from_slice::<Value>(&fixture).unwrap()
    );
    assert_eq!(evidence["paper"], "white");
    assert_eq!(evidence["preview_theme_applied"], false);
    assert_eq!(evidence["standalone_pdf"], false);
    assert_eq!(
        evidence["operators"].as_array().unwrap().len(),
        stream.operators().len()
    );
}
#[test]
fn unsupported_precision_paint_and_path_state_fail_without_rounding() {
    let stream = PdfCommandStream::from_mixed_replay(
        include_bytes!("fixtures/synthetic-mixed.json"),
        StreamLimits::default(),
    )
    .unwrap();
    assert_eq!(
        stream.content_bytes().unwrap_err(),
        PdfStreamError::NonTerminatingDecimal
    );
    assert!(stream.evidence_bytes().is_ok());
    let fixture = finite_fixture();
    let v: Value = serde_json::from_slice(&fixture).unwrap();
    let mut alpha = v.clone();
    alpha["primitives"][0]["paint"]["a"] = json!(0.5);
    assert!(matches!(
        PdfCommandStream::from_mixed_replay(
            &serde_json::to_vec(&alpha).unwrap(),
            StreamLimits::default()
        ),
        Err(PdfStreamError::Unsupported(_))
    ));
    let mut path = v;
    path["primitives"][0]["geometry"]["commands"][0] = json!(["line", point(0, 0)]);
    assert!(matches!(
        PdfCommandStream::from_mixed_replay(
            &serde_json::to_vec(&path).unwrap(),
            StreamLimits::default()
        ),
        Err(PdfStreamError::PathState)
    ));
    assert!(PdfCommandStream::from_mixed_replay(
        &fixture,
        StreamLimits {
            max_operators: 10,
            ..Default::default()
        }
    )
    .is_err());
    assert!(PdfCommandStream::from_mixed_replay(
        &fixture,
        StreamLimits {
            max_bytes: 10,
            ..Default::default()
        }
    )
    .is_err());
}
#[test]
fn decimal_encoder_is_exact_bounded_and_never_exponential() {
    assert_eq!(decimal(q(1, 8), 64).unwrap(), "0.125");
    assert_eq!(decimal(q(-21, 40), 64).unwrap(), "-0.525");
    assert_eq!(decimal(q(42, 1), 64).unwrap(), "42");
    assert_eq!(
        decimal(q(1, 3), 64),
        Err(PdfStreamError::NonTerminatingDecimal)
    );
    assert_eq!(decimal(q(1, 8), 2), Err(PdfStreamError::DecimalPrecision));
    for n in -20..=20 {
        for d in [1, 2, 4, 5, 8, 10, 16, 20, 25, 40, 100] {
            let value = q(n, d);
            let encoded = decimal(value, 64).unwrap();
            let negative = encoded.starts_with('-');
            let unsigned = encoded.trim_start_matches('-');
            let parts: Vec<_> = unsigned.split('.').collect();
            let places = parts.get(1).map_or(0, |s| s.len());
            let raw = parts.join("").parse::<i128>().unwrap();
            let recovered = q(if negative { -raw } else { raw }, 10u128.pow(places as u32));
            assert_eq!(recovered, value);
        }
    }
}
#[test]
fn shaped_stream_retains_original_gids_and_source_hash() {
    let bytes = include_bytes!("fixtures/synthetic-shaped.json");
    let stream = PdfCommandStream::from_shaped_replay(
        bytes,
        PdfPage {
            width: flashtex_rendering_core::Tick(1000000),
            height: flashtex_rendering_core::Tick(1000000),
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
    assert_eq!(stream.spans().len(), 3);
    assert_eq!(stream.source_sha256(), digest(bytes));
    let evidence: Value = serde_json::from_slice(&stream.evidence_bytes().unwrap()).unwrap();
    assert_eq!(
        evidence["original_fixture"]["clusters"][2]["glyphs"],
        json!([])
    );
    assert_eq!(
        evidence["original_fixture"]["identity"]["font_sha256"],
        serde_json::from_slice::<Value>(bytes).unwrap()["identity"]["font_sha256"]
    );
    assert_eq!(
        stream.content_bytes().unwrap_err(),
        PdfStreamError::NonTerminatingDecimal
    );
}
