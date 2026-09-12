//! Acceptance tests for FT-009: page dimensions, text placement, multiline
//! output, Unicode/font behaviour, structural validity, and (on macOS) that the
//! system's own PDF reader opens the fixture output.

use flashtex_pdf::verify::{check_structure, placements, rules, stream_data};
use flashtex_pdf::{CompileResult, Item, Page, PdfError, render_envelope, render_pdf};

const FIXTURE: &str = include_str!("../../../protocol/fixtures/compile-result.json");
/// Exact output of `flashtex-compiler` at de1020c for
/// `tests/fixtures/math-compile-request.json` (`$\\frac{a}{b}+\\alpha+\\sqrt{x}$`),
/// the reproduction from GitHub issue #9.
const MATH_RESULT: &str = include_str!("fixtures/math-compile-result.json");

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
    // catalog, pages, two fonts, info, one page, one content stream
    assert_eq!(s.object_count, 7);
    assert_eq!(s.stream_objects, vec![7]);

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
    assert!(find(&out.bytes, b"/BaseFont /Symbol >>").is_some());
    assert!(find(&out.bytes, b"/Font << /F1 3 0 R /F2 4 0 R >>").is_some());

    let content = stream_data(&out.bytes, 7).unwrap();
    let placed = placements(&content).unwrap();
    assert_eq!(placed.len(), 1);
    assert_eq!(placed[0].font, "F1");
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
    assert_eq!(s.object_count, 9);
    assert_eq!(s.stream_objects, vec![7, 9]);
    assert!(find(&out.bytes, b"/Count 2 ").is_some());
    assert!(find(&out.bytes, b"/Kids [ 6 0 R 8 0 R ]").is_some());

    let p6 = find(&out.bytes, b"\n6 0 obj\n").unwrap();
    let p8 = find(&out.bytes, b"\n8 0 obj\n").unwrap();
    let obj6 = &out.bytes[p6..p8];
    assert!(find(obj6, b"/MediaBox [ 0 0 612 792 ]").is_some());
    assert!(find(obj6, b"/Contents 7 0 R").is_some());
    let obj8 = &out.bytes[p8..];
    assert!(find(obj8, b"/MediaBox [ 0 0 595.28 841.89 ]").is_some());
    assert!(find(obj8, b"/Contents 9 0 R").is_some());

    let second = placements(&stream_data(&out.bytes, 9).unwrap()).unwrap();
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
    let placed = placements(&stream_data(&out.bytes, 7).unwrap()).unwrap();
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
                item("中 x dx 😀", 72.0, 100.0, 12.0),
                item("(paren) back\\slash", 72.0, 120.0, 12.0),
            ],
        )],
    };
    let out = render_pdf(&result).unwrap();
    check_structure(&out.bytes).unwrap();
    let placed = placements(&stream_data(&out.bytes, 7).unwrap()).unwrap();
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
    assert!(w.contains("U+4E2D"), "{w}");
    assert!(w.contains("U+1F600"), "{w}");
    assert!(w.contains("WinAnsi"), "{w}");
    assert!(w.contains("Symbol"), "{w}");

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
    let content = stream_data(&out.bytes, 7).unwrap();
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
fn issue_9_math_repro_renders_rule_greek_and_radical_without_warnings() {
    let out = render_envelope(MATH_RESULT).expect("math result renders");
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    check_structure(&out.bytes).unwrap();
    let content = stream_data(&out.bytes, 7).unwrap();

    // The fraction bar "──" (2 × U+2500 at 8.4pt, x=72, baseline 84.72) is a
    // filled rectangle, not glyphs: width 2 × 0.5 × 8.4 = 8.4pt, thickness
    // 0.06/0.7 × 8.4 = 0.72pt, bottom edge on the baseline.
    let bars = rules(&content).unwrap();
    assert_eq!(bars.len(), 1, "{bars:?}");
    let bar = &bars[0];
    assert_eq!(bar.x, 72.0);
    assert!((bar.width - 8.4).abs() < 0.0005, "{bar:?}");
    assert!((bar.height - 0.72).abs() < 0.0005, "{bar:?}");
    assert!((bar.y - (792.0 - 84.72)).abs() < 0.0005, "{bar:?}");
    assert!(find(&content, "─".as_bytes()).is_none(), "no U+2500 glyphs");
    assert!(find(&content, b"?").is_none(), "nothing substituted");

    // Text items, in compiler order, minus the rule: a b + α + √ x.
    let placed = placements(&content).unwrap();
    let summary: Vec<(&str, Vec<u8>)> = placed
        .iter()
        .map(|p| (p.font.as_str(), p.bytes.clone()))
        .collect();
    assert_eq!(
        summary,
        vec![
            ("F1", b"a".to_vec()),
            ("F1", b"b".to_vec()),
            ("F1", b"+".to_vec()),
            ("F2", vec![0x61]), // alpha in Symbol
            ("F1", b"+".to_vec()),
            ("F2", vec![0xD6]), // radical in Symbol
            ("F1", b"x".to_vec()),
        ]
    );
    let alpha = &placed[3];
    assert_eq!(alpha.font_size, 12.0);
    assert_eq!(alpha.x, 87.17);
    assert!((alpha.y - (792.0 - 87.36)).abs() < 0.0005);
    let radical = &placed[5];
    assert_eq!(radical.x, 99.94);
    assert!(placed.iter().all(|p| !p.continues));
    assert!(
        placed
            .iter()
            .filter(|p| p.font == "F1")
            .all(|p| p.font_size == 12.0 || (p.font_size - 8.4).abs() < 1e-9)
    );
}

#[test]
fn mixed_item_switches_fonts_inside_one_text_object() {
    let result = CompileResult {
        pages: vec![page(
            1,
            612.0,
            792.0,
            vec![
                item("x∈ℝ→∞", 72.0, 84.0, 12.0),
                item("─", 72.0, 100.0, 10.0),
            ],
        )],
    };
    let out = render_pdf(&result).unwrap();
    check_structure(&out.bytes).unwrap();
    assert!(find(&out.bytes, b"/Font << /F1 3 0 R /F2 4 0 R >>").is_some());
    let content = stream_data(&out.bytes, 7).unwrap();

    // ℝ (U+211D) is in neither font: substituted and reported; the rest of
    // the item still switches between Times and Symbol within one BT block.
    assert_eq!(out.warnings.len(), 1, "{:?}", out.warnings);
    assert!(out.warnings[0].contains("U+211D"));
    let placed = placements(&content).unwrap();
    let runs: Vec<(&str, bool, Vec<u8>)> = placed
        .iter()
        .map(|p| (p.font.as_str(), p.continues, p.bytes.clone()))
        .collect();
    assert_eq!(
        runs,
        vec![
            ("F1", false, b"x".to_vec()),
            ("F2", true, vec![0xCE]),
            ("F1", true, b"?".to_vec()),
            ("F2", true, vec![0xAE, 0xA5]),
        ]
    );
    assert!(placed.iter().all(|p| p.x == 72.0 && p.y == 708.0));
    assert_eq!(count(&content, b"BT\n"), 1);
    assert_eq!(count(&content, b" Tf\n"), 4);

    // A single dash is still a rule: 0.5 em wide, 0.06/0.7 em thick.
    let bars = rules(&content).unwrap();
    assert_eq!(bars.len(), 1);
    assert!((bars[0].width - 5.0).abs() < 0.0005);
    assert!((bars[0].height - 10.0 * 0.06 / 0.7).abs() < 0.0005);
    assert_eq!(bars[0].y, 692.0);
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

// ---------------------------------------------------------------------------
// Opt-in font embedding. These tests need a real TrueType font; they use the
// same discovery as the CLI (`FLASHTEX_UNICODE_FONT`, then macOS system fonts)
// and skip with a message when none exists.

fn embed_font_or_skip(test: &str) -> Option<flashtex_pdf::embed::EmbedFont> {
    match flashtex_pdf::embed::EmbedFont::discover() {
        Ok(Some(f)) => Some(f),
        Ok(None) => {
            eprintln!(
                "SKIPPED {test}: no TrueType font found (set {} or run on macOS)",
                flashtex_pdf::embed::ENV_VAR
            );
            None
        }
        Err(e) => panic!("font discovery failed: {e}"),
    }
}

/// Characters outside WinAnsi and Symbol: Cyrillic (in every candidate
/// font), CJK and double-struck R (coverage varies), and an emoji (in none).
const EMBED_TEXT: &str = "ж中ℝ😀";

#[test]
fn embedded_subset_font_covers_unicode_and_reports_the_rest() {
    use flashtex_pdf::embed::parse_to_unicode;
    use flashtex_pdf::truetype::{TrueTypeFont, verify_checksums};
    let Some(font) = embed_font_or_skip("embedded_subset_font_covers_unicode_and_reports_the_rest")
    else {
        return;
    };
    let result = CompileResult {
        pages: vec![page(
            1,
            612.0,
            792.0,
            vec![
                item("plain", 72.0, 84.0, 12.0),
                item(EMBED_TEXT, 72.0, 100.0, 12.0),
            ],
        )],
    };
    let options = flashtex_pdf::RenderOptions {
        embed_font: Some(font.clone()),
    };
    let out = flashtex_pdf::render_pdf_with(&result, &options).unwrap();
    let s = check_structure(&out.bytes).unwrap();
    // 7 base objects + 5 font objects; FontFile2 and ToUnicode are streams.
    assert_eq!(s.object_count, 12);
    assert_eq!(s.stream_objects, vec![7, 11, 12]);
    assert!(find(&out.bytes, b"/Subtype /Type0 /BaseFont /").is_some());
    assert!(find(&out.bytes, b"/Encoding /Identity-H").is_some());
    assert!(find(&out.bytes, b"/Subtype /CIDFontType2").is_some());
    assert!(find(&out.bytes, b"/CIDToGIDMap /Identity").is_some());
    assert!(find(&out.bytes, b"/FontFile2 11 0 R").is_some());
    assert!(find(&out.bytes, b"/ToUnicode 12 0 R").is_some());
    assert!(find(&out.bytes, b"/Font << /F1 3 0 R /F2 4 0 R /F3 8 0 R >>").is_some());

    // Every character either went through /F3 with a ToUnicode entry mapping
    // its glyph id back to it, or is named in a warning. Nothing vanishes.
    let to_unicode = parse_to_unicode(&stream_data(&out.bytes, 12).unwrap()).unwrap();
    let placed = placements(&stream_data(&out.bytes, 7).unwrap()).unwrap();
    let mut embedded_chars = Vec::new();
    for p in placed.iter().filter(|p| p.font == "F3") {
        for gid in p.bytes.chunks(2) {
            let gid = u16::from_be_bytes([gid[0], gid[1]]);
            assert_ne!(gid, 0, "never write .notdef");
            embedded_chars.push(
                *to_unicode
                    .get(&gid)
                    .unwrap_or_else(|| panic!("gid {gid} has no ToUnicode entry")),
            );
        }
    }
    let warned = out.warnings.join("\n");
    for c in EMBED_TEXT.chars() {
        let code = format!("U+{:04X}", c as u32);
        assert!(
            embedded_chars.contains(&c) || warned.contains(&code),
            "{c:?} neither embedded nor warned about; warnings: {warned}"
        );
    }
    assert!(
        embedded_chars.contains(&'ж'),
        "Cyrillic must be in any candidate font"
    );
    assert!(
        !embedded_chars.contains(&'😀'),
        "no candidate font has colour emoji outlines"
    );
    assert!(warned.contains("U+1F600"), "{warned}");
    assert!(warned.contains(&font.font.postscript_name), "{warned}");
    // Substituted characters are still written as '?' in a Times run.
    let substituted: Vec<u8> = placed
        .iter()
        .filter(|p| p.font == "F1" && p.y == 692.0)
        .flat_map(|p| p.bytes.clone())
        .collect();
    assert!(substituted.iter().all(|&b| b == b'?'), "{substituted:?}");
    assert_eq!(
        substituted.len(),
        EMBED_TEXT.chars().count() - embedded_chars.len()
    );
    assert_eq!(placed[0].font, "F1");
    assert_eq!(placed[0].bytes, b"plain");

    // The embedded program parses back as a consistent TrueType font whose
    // glyph count is exactly the used glyphs plus .notdef plus the components
    // composites pulled in, with valid table checksums.
    let program = stream_data(&out.bytes, 11).unwrap();
    verify_checksums(&program).unwrap();
    let parsed = TrueTypeFont::parse(program.clone()).unwrap();
    let subset = font.subset_for(EMBED_TEXT.chars()).unwrap();
    assert_eq!(program, subset.subset.bytes, "deterministic subset");
    assert_eq!(parsed.num_glyphs(), subset.subset.num_glyphs());
    assert_eq!(subset.chars.len(), embedded_chars.len());
    assert!(parsed.num_glyphs() as usize > embedded_chars.len());
    assert_eq!(parsed.units_per_em, font.font.units_per_em);
    let composites = parsed.num_glyphs() as usize - embedded_chars.len() - 1;
    eprintln!(
        "embedded {} from {}: {} glyphs ({} used + .notdef + {composites} composite parts), {} bytes",
        font.font.postscript_name,
        font.source.display(),
        parsed.num_glyphs(),
        embedded_chars.len(),
        program.len()
    );
    for (&c, &gid) in &subset.chars {
        assert_eq!(to_unicode.get(&gid), Some(&c));
        let original = font.font.glyph_id(c).unwrap();
        assert_eq!(parsed.advance(gid), font.font.advance(original));
    }
    assert!(
        program.len() < 60_000,
        "subset should be small, got {}",
        program.len()
    );
}

#[test]
fn embedding_is_off_by_default_and_a_bad_font_path_is_an_error() {
    let result = CompileResult {
        pages: vec![page(1, 612.0, 792.0, vec![item("ж", 72.0, 84.0, 12.0)])],
    };
    let out = render_pdf(&result).unwrap();
    assert!(find(&out.bytes, b"/Type0").is_none());
    assert!(find(&out.bytes, b"/F3").is_none());
    assert_eq!(out.warnings.len(), 1);
    assert!(out.warnings[0].contains("U+0436"));

    let missing =
        flashtex_pdf::embed::EmbedFont::load(std::path::Path::new("/nonexistent/font.ttf"));
    assert!(missing.is_err());
    let not_a_font =
        std::env::temp_dir().join(format!("flashtex-pdf-notafont-{}.ttf", std::process::id()));
    std::fs::write(&not_a_font, b"OTTO this is not really a font").unwrap();
    let err = flashtex_pdf::embed::EmbedFont::load(&not_a_font).unwrap_err();
    assert!(err.contains("OTTO"), "{err}");
    let _ = std::fs::remove_file(&not_a_font);
}

#[cfg(target_os = "macos")]
#[test]
fn cli_embed_font_writes_a_pdf_that_sips_opens() {
    let Some(font) = embed_font_or_skip("cli_embed_font_writes_a_pdf_that_sips_opens") else {
        return;
    };
    let exe = env!("CARGO_BIN_EXE_flashtex-pdf");
    let dir = std::env::temp_dir().join(format!("flashtex-pdf-embed-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let input = dir.join("unicode.json");
    let json = format!(
        r#"{{"protocol_version":1,"id":"u","type":"compile_result","payload":{{"pages":[{{"number":1,"width_pt":612,"height_pt":792,"items":[{{"kind":"text","text":"{EMBED_TEXT}","x_pt":72,"baseline_y_pt":84,"font_size_pt":12}}]}}]}}}}"#
    );
    std::fs::write(&input, json).unwrap();
    let out = dir.join("unicode.pdf");
    let output = std::process::Command::new(exe)
        .arg(&input)
        .arg("--out")
        .arg(&out)
        .arg("--verify")
        .arg("--embed-font")
        .arg(&font.source)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    assert!(stderr.contains("note: embedding subset of"), "{stderr}");
    assert!(
        stderr.contains("warning:") && stderr.contains("U+1F600"),
        "{stderr}"
    );
    assert!(
        stderr.contains("warning(s); the PDF was written"),
        "{stderr}"
    );

    let sips = std::process::Command::new("/usr/bin/sips")
        .args(["-g", "pixelWidth", "-g", "pixelHeight"])
        .arg(&out)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&sips.stdout);
    assert!(
        sips.status.success(),
        "{stdout}{}",
        String::from_utf8_lossy(&sips.stderr)
    );
    assert!(stdout.contains("pixelWidth: 612"), "{stdout}");
    let _ = std::fs::remove_dir_all(&dir);
}
