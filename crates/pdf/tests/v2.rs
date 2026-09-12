//! rendering-v2 display list → exact export (`flashtex_pdf::v2`).
//!
//! `tests/fixtures/v2-plain-paragraph.json` is the unmodified `display_list`
//! envelope `flashtex-render --v2 --secnumdepth 0` (branch
//! `agent/mac-render-pipeline/unified` at `ba5611f`) wrote for the visual-corpus
//! fixture `01-plain-paragraph` (body only). It references Latin Modern
//! `lmroman12-regular.otf` by content hash; tests that need the bytes skip
//! loudly when the font is not installed.

use flashtex_pdf::exact::{self, Op, SubsetOutcome};
use flashtex_pdf::reader::PdfFile;
use flashtex_pdf::sha256;
use flashtex_pdf::truetype::TrueTypeFont;
use flashtex_pdf::v2::{self, HashForm, V2Options};
use flashtex_pdf::verify;
use std::path::{Path, PathBuf};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/v2-plain-paragraph.json"
);

fn lm12() -> Option<PathBuf> {
    v2::font_dirs(&V2Options::default())
        .into_iter()
        .map(|d| d.join("lmroman12-regular.otf"))
        .find(|p| p.is_file())
}

fn ops_text(pdf: &[u8], object: usize) -> String {
    String::from_utf8_lossy(&verify::stream_data(pdf, object).unwrap()).into_owned()
}

#[test]
fn real_pipeline_envelope_exports_glyphs_by_original_gid_at_exact_positions() {
    let Some(font_path) = lm12() else {
        eprintln!("skipped: Latin Modern 12 not installed");
        return;
    };
    let (doc, report) = v2::from_v2_file(Path::new(FIXTURE), &V2Options::default()).unwrap();
    assert_eq!(report.pages, 1);
    assert_eq!(report.runs, 13);
    assert_eq!(report.glyphs, 55);
    assert_eq!(report.rules, 0);
    assert_eq!(report.fonts.len(), 1);
    let f = &report.fonts[0];
    assert_eq!(f.resource, "F1");
    assert_eq!(f.postscript_name, "LMRoman12-Regular");
    assert_eq!(f.path, font_path);
    assert_eq!(f.outcome, SubsetOutcome::CffSubset);
    assert_eq!(f.glyphs, 18);
    assert_eq!(
        f.hash_form,
        HashForm::BytesAndFaceIndex,
        "flashtex-render emits font-engine's content hash (bytes || face index)"
    );
    assert!(
        report
            .notes
            .iter()
            .any(|n| n.contains("render-pipeline deviation"))
    );
    assert!(report.diagnostics.is_empty());

    let out = exact::render_exact(&doc).unwrap();
    verify::check_structure(&out.bytes).unwrap();
    let content = ops_text(&out.bytes, 5);
    // 12 TeX pt = 12535902 ticks: the exact decimal, not pdfTeX's 11.9552.
    assert!(
        content.contains("/F1 11.9551677703857421875 Tf\n"),
        "{content}"
    );
    // First glyph 'H' (LM GID 62) at origin 75497472 ticks = 72 bp exactly,
    // baseline 88033374 ticks below the top of a 830472192-tick page.
    assert!(
        content.contains("1 0 0 1 72 708.0448322296142578125 Tm\n(\\000>) Tj\n"),
        "{content}"
    );
    // Every glyph carries its own Tm: 12 TeX pt is 12535902 ticks, which has
    // a factor of 3, so no gap between consecutive origins is an exactly
    // representable TJ adjustment and nothing joins or kerns.
    assert_eq!((report.joined_glyphs, report.kerned_glyphs), (0, 0));
    assert_eq!(content.matches(" Tm\n").count(), 55);
    let ops = exact::parse(content.as_bytes()).unwrap();
    assert_positions_round_trip(&ops, &doc, FIXTURE);
    assert_eq!(
        ops.iter().filter(|o| matches!(o, Op::BeginText)).count(),
        13
    );
    // The embedded program is a CID-keyed subset of the real file, GIDs kept.
    let file = PdfFile::parse(&out.bytes).unwrap();
    let page = file.pages().unwrap()[0];
    let fonts = file.page_fonts(page);
    assert_eq!(fonts.len(), 1);
    let re = flashtex_pdf::compare::font_from_dict(&file, fonts["F1"]).unwrap();
    let exact::ExactFont::CidCff(cid) = re else {
        panic!("expected CIDFontType0C")
    };
    let sub = flashtex_pdf::cff::CffFont::parse(cid.program.bytes()).unwrap();
    let src = TrueTypeFont::load(&font_path).unwrap();
    let src_cff = flashtex_pdf::cff::CffFont::parse(src.cff_table().unwrap()).unwrap();
    assert!(sub.is_cid_keyed());
    // Subset glyph i carries CID = original GID and the original charstring.
    let mut used: Vec<u16> = cid.widths.keys().copied().collect();
    used.sort_unstable();
    assert_eq!(used.len(), 18);
    for (i, &gid) in std::iter::once(&0u16).chain(used.iter()).enumerate() {
        assert_eq!(sub.charset_entry(i as u16), Some(gid));
        assert_eq!(
            sub.expanded_charstring(i as u16).unwrap(),
            src_cff.expanded_charstring(gid).unwrap()
        );
    }
    let tu = exact::parse_to_unicode(cid.to_unicode_verbatim.as_deref().unwrap()).unwrap();
    assert_eq!(tu.get(&62).map(String::as_str), Some("H"));
    assert!(
        tu.values().any(|t| t == "fi"),
        "the fi ligature's cluster text reaches ToUnicode: {tu:?}"
    );
    // Deterministic.
    let (doc2, _) = v2::from_v2_file(Path::new(FIXTURE), &V2Options::default()).unwrap();
    assert_eq!(exact::render_exact(&doc2).unwrap().bytes, out.bytes);
}

/// A hand-built envelope over Latin Modern 12: the plain SHA-256(bytes) hash
/// form, a glyph that continues by hmtx advance, one that does not, a rule
/// and a coloured run.
#[test]
fn hand_built_envelope_joins_by_hmtx_advance_and_converts_rules_and_colour() {
    let Some(font_path) = lm12() else {
        eprintln!("skipped: Latin Modern 12 not installed");
        return;
    };
    let bytes = std::fs::read(&font_path).unwrap();
    let font = TrueTypeFont::load(&font_path).unwrap();
    let sha = sha256::hex(&bytes);
    let gid_h = font.glyph_id('H').unwrap();
    let gid_e = font.glyph_id('e').unwrap();
    // A size whose tick count is a multiple of 1000 so that hmtx advances
    // (1000/em) scale to whole ticks: 12,500,000 ticks = 11.920928955078125 bp.
    let size: i64 = 12_500_000;
    let adv_h = font.advance(gid_h) as i64;
    assert_eq!(font.units_per_em, 1000);
    let adv_ticks = adv_h * size / 1000;
    assert_eq!(adv_ticks * 1000, adv_h * size, "exact");
    let x0: i64 = 72 << 20;
    let y: i64 = 100 << 20;
    let envelope = format!(
        r#"{{"protocol_version":2,"id":"t","type":"display_list","payload":{{
        "render_format":"display-list-v2","coordinate_unit":"bp_2pow20","color_space":"srgb","text_extraction":"cluster-actualtext",
        "project_id":"t","revision":1,"required_features":["glyph_run","rule"],"documents":[],
        "fonts":[{{"font_id":"{sha}","sha256":"{sha}","byte_length":{len},"format":"opentype-cff","face_index":0,"units_per_em":1000,"glyph_count":{gc},"postscript_name":"LMRoman12-Regular"}}],
        "pages":[{{"number":1,"width":{pw},"height":{ph},"items":[
          {{"kind":"glyph_run","font_id":"{sha}","font_size":{size},"text":"HeH","paint":{{"r":0,"g":0,"b":0,"a":1}},
           "glyphs":[
             {{"gid":{gh},"origin_x":{x0},"baseline_y":{y},"advance_x":{adv},"advance_y":0,"cluster":0}},
             {{"gid":{ge},"origin_x":{x1},"baseline_y":{y},"advance_x":1000,"advance_y":0,"cluster":1}},
             {{"gid":{gh},"origin_x":{x2},"baseline_y":{y},"advance_x":{adv},"advance_y":0,"cluster":2}}],
           "clusters":[{{"text_start_byte":0,"text_end_byte":1}},{{"text_start_byte":1,"text_end_byte":2}},{{"text_start_byte":2,"text_end_byte":3}}]}},
          {{"kind":"rule","x":{x0},"top":{rt},"width":{rw},"height":{rh},"paint":{{"r":0,"g":0,"b":0,"a":1}}}},
          {{"kind":"glyph_run","font_id":"{sha}","font_size":{size},"text":"e","paint":{{"r":0.5,"g":0,"b":1,"a":1}},
           "glyphs":[{{"gid":{ge},"origin_x":{x0},"baseline_y":{y2},"advance_x":0,"advance_y":0,"cluster":0}}],
           "clusters":[{{"text_start_byte":0,"text_end_byte":1}}]}}
        ]}}],"diagnostics":[]}}}}"#,
        len = bytes.len(),
        gc = font.num_glyphs(),
        pw = 612i64 << 20,
        ph = 792i64 << 20,
        gh = gid_h,
        ge = gid_e,
        adv = adv_ticks,
        x1 = x0 + adv_ticks,
        x2 = x0 + adv_ticks + 1000 + 5, // not hmtx-continued
        rt = 110i64 << 20,
        rw = 3 << 19, // 1.5 bp
        rh = 1 << 18, // 0.25 bp
        y2 = 200i64 << 20,
    );
    let dir = std::env::temp_dir().join(format!("flashtex-pdf-v2-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::copy(&font_path, dir.join("lm.otf")).unwrap();
    let options = V2Options {
        font_dirs: vec![dir.clone()],
    };
    let (doc, report) = v2::from_v2(&envelope, &options).unwrap();
    assert_eq!(report.fonts[0].hash_form, HashForm::Bytes);
    assert_eq!(report.fonts[0].path, dir.join("lm.otf"));
    assert!(report.notes.is_empty(), "{:?}", report.notes);
    assert_eq!(
        (report.joined_glyphs, report.kerned_glyphs),
        (1, 1),
        "e continues at the natural advance; the second H needs an exact kern"
    );
    let out = exact::render_exact(&doc).unwrap();
    verify::check_structure(&out.bytes).unwrap();
    let content = ops_text(&out.bytes, 5);
    // The second H sits 1005 ticks past e's natural advance (e is 435/1000
    // wide): n = 435 - 1000 * 1005 / 12,500,000 = 434.9196, exactly.
    let expect = format!(
        "BT\n/F1 11.920928955078125 Tf\n1 0 0 1 72 692 Tm\n[(\\000{h}\\000{e})434.9196(\\000{h})] TJ\nET\n72 {ry} 1.5 0.25 re\nf\nq\n0.5 0 1 rg\nBT\n/F1 11.920928955078125 Tf\n1 0 0 1 72 592 Tm\n(\\000{e}) Tj\nET\nQ\n",
        h = gid_h as u8 as char,
        e = gid_e as u8 as char,
        ry = v2::bp(((792i64 << 20) - (110i64 << 20) - (1 << 18)) as i128).unwrap(),
    );
    assert_eq!(
        font.advance(gid_e),
        435,
        "the expectation above assumes e's advance"
    );
    // Codes below 32 or above 126 are octal-escaped by the writer; compare
    // through the parser instead of raw text where GIDs are small.
    let got = exact::parse(content.as_bytes()).unwrap();
    let want = exact::parse(expect.as_bytes()).unwrap();
    assert_eq!(got, want, "content:\n{content}\nexpected:\n{expect}");
    std::fs::write(dir.join("list.json"), &envelope).unwrap();
    assert_positions_round_trip(&got, &doc, dir.join("list.json").to_str().unwrap());
    // Deterministic bytes.
    assert_eq!(exact::render_exact(&doc).unwrap().bytes, out.bytes);
    let _ = std::fs::remove_dir_all(&dir);
}

/// Replays the written text operators exactly and checks every glyph lands
/// on the envelope's origin (`origin_x / 2^20`, `(height - baseline_y) / 2^20`).
fn assert_positions_round_trip(ops: &[Op], doc: &exact::ExactDocument, envelope_path: &str) {
    use exact::{ExactFont, Ratio};
    let width = |font: &str, code: u16| -> Option<Ratio> {
        match doc.fonts.get(font)? {
            ExactFont::CidCff(c) | ExactFont::CidTrueType(c) => Some(Ratio::from_decimal(
                c.widths.get(&code).unwrap_or(&c.default_width),
            )),
            ExactFont::Simple(_) => None,
        }
    };
    let positions = exact::glyph_positions(ops, &|_| true, &width).unwrap();
    // Expected origins straight from the envelope, in page order.
    let text = std::fs::read_to_string(envelope_path).unwrap();
    let json = flashtex_pdf::json::parse(&text).unwrap();
    let mut expected: Vec<(u16, Ratio, Ratio)> = Vec::new();
    let page = &json
        .get("payload")
        .unwrap()
        .get("pages")
        .unwrap()
        .as_array()
        .unwrap()[0];
    let height = page.get("height").unwrap().as_f64().unwrap() as i128;
    for item in page.get("items").unwrap().as_array().unwrap() {
        if item.get("kind").unwrap().as_str() != Some("glyph_run") {
            continue;
        }
        for g in item.get("glyphs").unwrap().as_array().unwrap() {
            let gid = g.get("gid").unwrap().as_f64().unwrap() as u16;
            let ox = g.get("origin_x").unwrap().as_f64().unwrap() as i128;
            let by = g.get("baseline_y").unwrap().as_f64().unwrap() as i128;
            expected.push((
                gid,
                Ratio::new(ox, 1 << 20),
                Ratio::new(height - by, 1 << 20),
            ));
        }
    }
    assert_eq!(positions.len(), expected.len());
    for (p, (gid, x, y)) in positions.iter().zip(&expected) {
        assert_eq!(p.code, *gid);
        assert_eq!(
            (p.x, p.y),
            (*x, *y),
            "glyph {gid} replays to its envelope origin"
        );
    }
}

#[test]
fn unsupported_envelope_content_is_refused_not_approximated() {
    let base = r#"{"protocol_version":2,"id":"t","type":"display_list","payload":{"render_format":"display-list-v2","coordinate_unit":"bp_2pow20","color_space":"srgb","fonts":[],"pages":[{"number":1,"width":1048576,"height":1048576,"items":[ITEM]}],"diagnostics":[]}}"#;
    let cases = [
        (
            r#"{"kind":"image","paint":{"r":0,"g":0,"b":0,"a":1}}"#,
            "item kind \"image\"",
        ),
        (
            r#"{"kind":"rule","x":0,"top":0,"width":5,"height":5,"paint":{"r":0,"g":0,"b":0,"a":0.5}}"#,
            "alpha 0.5",
        ),
        (
            r#"{"kind":"rule","x":0,"top":0,"width":0,"height":5,"paint":{"r":0,"g":0,"b":0,"a":1}}"#,
            "not positive",
        ),
        (
            r#"{"kind":"rule","x":0.5,"top":0,"width":5,"height":5,"paint":{"r":0,"g":0,"b":0,"a":1}}"#,
            "not an integer tick",
        ),
        (
            r#"{"kind":"glyph_run","font_id":"nope","font_size":100,"text":"","paint":{"r":0,"g":0,"b":0,"a":1},"glyphs":[],"clusters":[]}"#,
            "not in payload.fonts",
        ),
    ];
    for (item, expect) in cases {
        let e = v2::from_v2(&base.replace("ITEM", item), &V2Options::default()).unwrap_err();
        assert!(e.contains(expect), "{item}: {e}");
    }
    let e = v2::from_v2(&base.replace("ITEM", r#"{"kind":"rule","x":0,"top":0,"width":5,"height":5,"paint":{"r":0.1,"g":0,"b":0,"a":1}}"#), &V2Options::default())
        .unwrap_err();
    assert!(e.contains("0.1"), "non-terminating colour is refused: {e}");
    // A missing font is a named error, not a fallback.
    let with_font = r#"{"protocol_version":2,"id":"t","type":"display_list","payload":{"render_format":"display-list-v2","coordinate_unit":"bp_2pow20","color_space":"srgb","fonts":[{"font_id":"0000000000000000000000000000000000000000000000000000000000000000","sha256":"0000000000000000000000000000000000000000000000000000000000000000","byte_length":7,"format":"opentype-cff","face_index":0,"units_per_em":1000,"glyph_count":10,"postscript_name":"Nope"}],"pages":[{"number":1,"width":1048576,"height":1048576,"items":[{"kind":"glyph_run","font_id":"0000000000000000000000000000000000000000000000000000000000000000","font_size":100,"text":"a","paint":{"r":0,"g":0,"b":0,"a":1},"glyphs":[{"gid":3,"origin_x":0,"baseline_y":0,"advance_x":0,"advance_y":0,"cluster":0}],"clusters":[{"text_start_byte":0,"text_end_byte":1}]}]}],"diagnostics":[]}}"#;
    let e = v2::from_v2(
        with_font,
        &V2Options {
            font_dirs: vec![std::env::temp_dir()],
        },
    )
    .unwrap_err();
    assert!(e.contains("Nope") && e.contains("was not found"), "{e}");
}
