//! Acceptance tests for FT-009: page dimensions, text placement, multiline
//! output, Unicode/font behaviour, structural validity, and (on macOS) that the
//! system's own PDF reader opens the fixture output.

use flashtex_pdf::verify::{check_structure, placements, stream_data};
use flashtex_pdf::{CompileResult, Item, Page, PdfError, render_envelope, render_pdf};

const FIXTURE: &str = include_str!("../../../protocol/fixtures/compile-result.json");

fn item(text: &str, x: f64, baseline: f64, size: f64) -> Item {
    Item {
        text: text.into(),
        x_pt: x,
        baseline_y_pt: baseline,
        font_size_pt: size,
    }
}

fn page(number: u32, w: f64, h: f64, items: Vec<Item>) -> Page {
    Page {
        number,
        width_pt: w,
        height_pt: h,
        items,
    }
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

fn count(hay: &[u8], needle: &[u8]) -> usize {
    hay.windows(needle.len()).filter(|w| *w == needle).count()
}

#[test]
fn fixture_renders_one_letter_page_with_valid_structure() {
    let out = render_envelope(FIXTURE).expect("fixture renders");
    assert!(
        out.warnings.is_empty(),
        "fixture is plain ASCII: {:?}",
        out.warnings
    );
    assert!(out.bytes.starts_with(b"%PDF-1.4\n"));
    assert!(out.bytes.ends_with(b"%%EOF\n"));

    let s = check_structure(&out.bytes).expect("xref offsets point at their objects");
    // catalog, pages, font, info, one page, one content stream
    assert_eq!(s.object_count, 6);
    assert_eq!(s.stream_objects, vec![6]);

    assert_eq!(
        count(&out.bytes, b"/Type /Page\n"),
        0,
        "no stray page markers"
    );
    assert_eq!(count(&out.bytes, b"/Type /Page "), 1);
    assert!(find(&out.bytes, b"/MediaBox [ 0 0 612 792 ]").is_some());
    assert!(find(&out.bytes, b"/Count 1 ").is_some());
    assert!(
        find(
            &out.bytes,
            b"/BaseFont /Times-Roman /Encoding /WinAnsiEncoding"
        )
        .is_some()
    );

    let content = stream_data(&out.bytes, 6).unwrap();
    let placed = placements(&content).unwrap();
    assert_eq!(placed.len(), 1);
    assert_eq!(placed[0].font_size, 12.0);
    assert_eq!((placed[0].x, placed[0].y), (72.0, 792.0 - 84.0));
    assert_eq!(placed[0].bytes, b"Hello FlashTeX.");
}

#[test]
fn two_synthetic_pages_get_their_own_mediabox_and_content() {
    let result = CompileResult {
        pages: vec![
            page(1, 612.0, 792.0, vec![item("first", 72.0, 84.0, 12.0)]),
            page(2, 595.28, 841.89, vec![item("second", 56.7, 70.9, 10.5)]),
        ],
    };
    let out = render_pdf(&result).unwrap();
    let s = check_structure(&out.bytes).unwrap();
    assert_eq!(s.object_count, 8);
    assert_eq!(s.stream_objects, vec![6, 8]);
    assert!(find(&out.bytes, b"/Count 2 ").is_some());
    assert!(find(&out.bytes, b"/Kids [ 5 0 R 7 0 R ]").is_some());

    let p5 = find(&out.bytes, b"\n5 0 obj\n").unwrap();
    let p7 = find(&out.bytes, b"\n7 0 obj\n").unwrap();
    let obj5 = &out.bytes[p5..p7];
    assert!(find(obj5, b"/MediaBox [ 0 0 612 792 ]").is_some());
    assert!(find(obj5, b"/Contents 6 0 R").is_some());
    let obj7 = &out.bytes[p7..];
    assert!(find(obj7, b"/MediaBox [ 0 0 595.28 841.89 ]").is_some());
    assert!(find(obj7, b"/Contents 8 0 R").is_some());

    let second = placements(&stream_data(&out.bytes, 8).unwrap()).unwrap();
    assert_eq!(second.len(), 1);
    assert_eq!(second[0].font_size, 10.5);
    assert_eq!(second[0].x, 56.7);
    assert!((second[0].y - (841.89 - 70.9)).abs() < 0.0005);
    assert_eq!(second[0].bytes, b"second");
}

#[test]
fn multiline_placement_maps_every_baseline_to_pdf_space() {
    // Three lines as the FT-002 layout would emit them: one item per word, a
    // 14.4pt leading, and a heading at a larger size.
    let items = vec![
        item("Title", 72.0, 89.0, 17.0),
        item("Hello", 72.0, 115.4, 12.0),
        item("world", 105.36, 115.4, 12.0),
        item("Second", 72.0, 129.8, 12.0),
        item("line", 114.36, 129.8, 12.0),
        item("Third", 72.0, 144.2, 12.0),
    ];
    let result = CompileResult {
        pages: vec![page(1, 612.0, 792.0, items.clone())],
    };
    let out = render_pdf(&result).unwrap();
    check_structure(&out.bytes).unwrap();
    let placed = placements(&stream_data(&out.bytes, 6).unwrap()).unwrap();
    assert_eq!(placed.len(), items.len());
    for (p, i) in placed.iter().zip(&items) {
        assert_eq!(p.font_size, i.font_size_pt, "{}", i.text);
        assert_eq!(p.x, i.x_pt, "{}", i.text);
        assert!(
            (p.y - (792.0 - i.baseline_y_pt)).abs() < 0.0005,
            "{}",
            i.text
        );
        assert_eq!(p.bytes, i.text.as_bytes());
    }
    let baselines: std::collections::BTreeSet<String> =
        placed.iter().map(|p| format!("{:.3}", p.y)).collect();
    assert_eq!(baselines.len(), 4, "four distinct baselines survive");
}

#[test]
fn winansi_characters_are_encoded_and_others_warn() {
    let result = CompileResult {
        pages: vec![page(
            1,
            612.0,
            792.0,
            vec![
                item("café — naïve", 72.0, 84.0, 12.0),
                item("∫ x dx 😀", 72.0, 100.0, 12.0),
                item("(paren) back\\slash", 72.0, 120.0, 12.0),
            ],
        )],
    };
    let out = render_pdf(&result).unwrap();
    check_structure(&out.bytes).unwrap();
    let placed = placements(&stream_data(&out.bytes, 6).unwrap()).unwrap();
    assert_eq!(placed.len(), 3);

    // é -> 0xE9, — -> 0x97, ï -> 0xEF: WinAnsi bytes, not UTF-8 sequences.
    assert_eq!(placed[0].bytes, b"caf\xE9 \x97 na\xEFve");
    assert!(
        find(&out.bytes, b"caf\xE9").is_some(),
        "byte 0xE9 appears literally"
    );
    assert!(
        find(&out.bytes, "café".as_bytes()).is_none(),
        "no UTF-8 leak"
    );

    // Unrepresentable characters become '?' and are reported, never dropped.
    assert_eq!(placed[1].bytes, b"? x dx ?");
    assert_eq!(out.warnings.len(), 1, "{:?}", out.warnings);
    let w = &out.warnings[0];
    assert!(w.contains("page 1"), "{w}");
    assert!(w.contains("item 1"), "{w}");
    assert!(w.contains("U+222B"), "{w}");
    assert!(w.contains("U+1F600"), "{w}");
    assert!(w.contains("WinAnsi"), "{w}");

    // Delimiters are escaped in the file and round-trip through the reader.
    assert_eq!(placed[2].bytes, b"(paren) back\\slash");
    assert!(find(&out.bytes, b"(\\(paren\\) back\\\\slash) Tj").is_some());
}

#[test]
fn unsupported_item_kinds_and_bad_envelopes_are_explicit() {
    let json = r#"{"protocol_version":1,"id":"x","type":"compile_result","payload":{"pages":[
        {"number":1,"width_pt":612,"height_pt":792,"items":[
            {"kind":"rule","x_pt":1,"y_pt":2},
            {"kind":"text","text":"ok","x_pt":72,"baseline_y_pt":84,"font_size_pt":12}
        ]}]}}"#;
    let out = render_envelope(json).unwrap();
    assert_eq!(out.warnings.len(), 1);
    assert!(out.warnings[0].contains("\"rule\""));
    assert!(out.warnings[0].contains("skipped"));

    let v9 = json.replace("\"protocol_version\":1", "\"protocol_version\":9");
    assert!(matches!(render_envelope(&v9), Err(PdfError::Protocol(_))));
    let wrong = json.replace(
        "\"type\":\"compile_result\"",
        "\"type\":\"capture_proposal\"",
    );
    assert!(matches!(
        render_envelope(&wrong),
        Err(PdfError::Protocol(_))
    ));
    assert!(matches!(
        render_envelope("not json"),
        Err(PdfError::Json(_))
    ));

    assert!(matches!(
        render_pdf(&CompileResult::default()),
        Err(PdfError::Invalid(_))
    ));
    let zero = CompileResult {
        pages: vec![page(1, 0.0, 792.0, vec![])],
    };
    assert!(matches!(render_pdf(&zero), Err(PdfError::Invalid(_))));
    let nan = CompileResult {
        pages: vec![page(1, 612.0, 792.0, vec![item("x", f64::NAN, 1.0, 12.0)])],
    };
    assert!(matches!(render_pdf(&nan), Err(PdfError::Invalid(_))));
}

#[test]
fn export_is_white_and_theme_independent() {
    // The writer has no theme input at all; the only colour operator it emits
    // is black text, and it never paints a background rectangle.
    let out = render_envelope(FIXTURE).unwrap();
    let content = stream_data(&out.bytes, 6).unwrap();
    assert!(
        content.starts_with(b"0 g\n"),
        "black non-stroking colour set explicitly"
    );
    assert_eq!(count(&content, b" g\n"), 1, "no other gray colour changes");
    for op in [b" rg\n".as_slice(), b" k\n", b" re\n", b" f\n", b" G\n"] {
        assert!(
            find(&content, op).is_none(),
            "no fill/rect/stroke operator {:?}",
            op
        );
    }
    assert!(find(&out.bytes, b"/Group").is_none());
}

#[test]
fn output_is_deterministic() {
    let a = render_envelope(FIXTURE).unwrap().bytes;
    let b = render_envelope(FIXTURE).unwrap().bytes;
    assert_eq!(a, b);
}

#[test]
fn cli_writes_fixture_pdf_and_rejects_unknown_type() {
    let exe = env!("CARGO_BIN_EXE_flashtex-pdf");
    let dir = std::env::temp_dir().join(format!("flashtex-pdf-cli-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let out = dir.join("fixture.pdf");

    let status = std::process::Command::new(exe)
        .arg("--out")
        .arg(&out)
        .arg("--verify")
        .stdin(std::fs::File::open(fixture_path()).unwrap())
        .status()
        .unwrap();
    assert!(status.success());
    let bytes = std::fs::read(&out).unwrap();
    assert_eq!(bytes, render_envelope(FIXTURE).unwrap().bytes);

    let bad = dir.join("bad.json");
    std::fs::write(&bad, FIXTURE.replace("compile_result", "compile")).unwrap();
    let output = std::process::Command::new(exe)
        .arg(&bad)
        .arg("--out")
        .arg(dir.join("bad.pdf"))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("\"compile\""), "{stderr}");
    assert!(!dir.join("bad.pdf").exists());

    let _ = std::fs::remove_dir_all(&dir);
}

fn fixture_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../protocol/fixtures/compile-result.json")
}

/// macOS ships a PDF reader in `sips`; if it can report pixel dimensions the
/// file parsed as a document, independent of this crate's own checks.
#[cfg(target_os = "macos")]
#[test]
fn macos_sips_opens_the_fixture_pdf() {
    let out = render_envelope(FIXTURE).unwrap();
    let dir = std::env::temp_dir().join(format!("flashtex-pdf-sips-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("fixture.pdf");
    std::fs::write(&path, &out.bytes).unwrap();

    let output = std::process::Command::new("/usr/bin/sips")
        .args(["-g", "pixelWidth", "-g", "pixelHeight"])
        .arg(&path)
        .output()
        .expect("sips exists on macOS");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "sips failed: {stdout}{stderr}");
    assert!(stdout.contains("pixelWidth: 612"), "{stdout}");
    assert!(stdout.contains("pixelHeight: 792"), "{stdout}");
    assert!(!stderr.contains("Error"), "{stderr}");

    let _ = std::fs::remove_dir_all(&dir);
}
