//! The compiler's control-word symbols (pin `49e6eb43`) through TeX's
//! metrics and Latin Modern Math: composites (`\neq` = `\not=`), the extra
//! cmsy slots (`\perp`), `\cdot`'s class, `\left`/`\right` fences re-derived
//! from the source, and the typed limitation for `\angle`.

mod common;

use common::*;
use flashtex_render_pipeline::display::Item;
use flashtex_render_pipeline::typeset::{convert_math, fence_before, fence_of, Fence};
use flashtex_math_layout::{AtomClass, Nucleus};

fn doc(body: &str) -> String {
    format!("\\begin{{document}}{body}\\end{{document}}")
}

/// Math glyph runs of page 1 as `(text, origin x bp)`.
fn math_words(body: &str) -> (Vec<(String, f64)>, Vec<String>) {
    let r = render_one(&doc(body));
    let mut out = Vec::new();
    for item in &r.v2.pages[0].items {
        if let Item::GlyphRun(run) = item {
            out.push((run.text.clone(), run.glyphs[0].origin_x.to_bp()));
        }
    }
    let diags = r.v1_diagnostics_codes();
    (out, diags)
}

trait Codes {
    fn v1_diagnostics_codes(&self) -> Vec<String>;
}

impl Codes for flashtex_render_pipeline::Rendered {
    fn v1_diagnostics_codes(&self) -> Vec<String> {
        self.v2.diagnostics.iter().map(|d| format!("{}:{}", d.code, d.message)).collect()
    }
}

#[test]
fn fences_are_recovered_from_the_source_bytes() {
    assert_eq!(fence_before("$\\left( x", 6), Some(Fence::Left));
    assert_eq!(fence_before("$\\left  ( x", 8), Some(Fence::Left));
    assert_eq!(fence_before("x \\right)", 8), Some(Fence::Right));
    assert_eq!(fence_before("x \\\\left(", 8), None, "an escaped backslash is not a control word");
    assert_eq!(fence_before("( x", 0), None);
    // Compiler pin 87df3e4a: the delimiter span starts at the control word.
    assert_eq!(fence_of("$\\left( x", 1), Some(Fence::Left));
    assert_eq!(fence_of("x \\right)", 2), Some(Fence::Right));
    assert_eq!(fence_of("$\\leftarrow", 1), None, "\\leftarrow is not a fence");
    assert_eq!(fence_of("$\\left( x", 6), Some(Fence::Left), "older pins: span at the delimiter");
}

#[test]
fn composite_and_extra_symbols_convert_with_texbook_classes() {
    use flashtex_compiler::math::{MathAtom, MathList, Nucleus as N};
    use flashtex_compiler::{DocumentId, Span};
    let sym = |s: &str| MathAtom { nucleus: N::Symbol(s.to_string()), span: Span::in_document(DocumentId(0), 0, 1), superscript: None, subscript: None };
    let list = MathList { atoms: ["a", "\u{2260}", "b", "\u{00B7}", "c", "\u{22A5}", "d", "\u{2209}", "e"].iter().map(|s| sym(s)).collect() };
    let list = convert_math(&list);
    let classes: Vec<(AtomClass, Option<char>)> = list
        .atoms
        .iter()
        .map(|a| (a.class, if let Nucleus::Symbol(c) = a.nucleus { Some(c) } else { None }))
        .collect();
    use AtomClass::*;
    assert_eq!(
        classes,
        vec![
            (Ord, Some('a')),
            (Rel, Some('\u{0338}')),
            (Rel, Some('=')),
            (Ord, Some('b')),
            (Bin, Some('\u{22C5}')),
            (Ord, Some('c')),
            (Rel, Some('\u{22A5}')),
            (Ord, Some('d')),
            (Rel, Some('\u{0338}')),
            (Rel, Some('\u{2208}')),
            (Ord, Some('e')),
        ]
    );
}

#[test]
fn every_compiler_symbol_typesets_without_a_missing_glyph() {
    if !lm_available() {
        return;
    }
    let mut body = String::from("$");
    for (name, _) in flashtex_compiler::math::COMMAND_GLYPHS {
        body.push_str(&format!("\\{name} "));
    }
    body.push('$');
    let (words, diags) = math_words(&body);
    assert!(!words.is_empty());
    let limitations: Vec<&String> = diags.iter().filter(|d| d.starts_with("math_limitation") || d.starts_with("missing_glyph")).collect();
    // Compiler pin 887bf21 (main) no longer lists \angle (a constructed
    // macro in fontmath.ltx, not a glyph), so every listed symbol has a glyph.
    assert_eq!(limitations.len(), 0, "{limitations:?}");
}

#[test]
fn not_equal_is_a_zero_width_slash_over_the_equals_sign() {
    if !lm_available() {
        return;
    }
    let (eq, _) = math_words("$a = b$");
    let (neq, _) = math_words("$a \\neq b$");
    // The slash is placed (it joins the preceding run as a combining
    // character) and `b` sits where it sits after `=`: the negation slash
    // has no width (cmsy 0x36).
    assert!(neq.iter().any(|(t, _)| t.contains('\u{0338}')), "{neq:?}");
    let b_eq = eq.last().unwrap().1;
    let b_neq = neq.last().unwrap().1;
    assert!((b_eq - b_neq).abs() < 1e-6, "b at {b_neq} vs {b_eq}");
}

#[test]
fn left_right_grows_the_delimiter_to_the_body() {
    if !lm_available() {
        return;
    }
    let plain = render_one(&doc("$( \\frac{a}{b} )$"));
    let fenced = render_one(&doc("$\\left( \\frac{a}{b} \\right)$"));
    let width = |r: &flashtex_render_pipeline::Rendered| -> f64 {
        let mut xs: Vec<f64> = Vec::new();
        for item in &r.v2.pages[0].items {
            if let Item::GlyphRun(run) = item {
                for g in &run.glyphs {
                    xs.push(g.origin_x.to_bp());
                }
            }
        }
        xs.iter().cloned().fold(f64::MIN, f64::max) - xs.iter().cloned().fold(f64::MAX, f64::min)
    };
    assert!(fenced.v2.diagnostics.iter().all(|d| d.code != "math_limitation"), "{:?}", fenced.v2.diagnostics);
    // A \left( around a text-style fraction selects a larger cmex variant
    // than the 12 pt roman parenthesis, so the closing delimiter moves right.
    assert!(width(&fenced) > width(&plain) + 0.5, "fenced {} vs plain {}", width(&fenced), width(&plain));
}

/// `(gid, ink height + depth in bp, ink top and bottom in bp relative to
/// the first text word's baseline: negative = above)` of every glyph drawn
/// from Latin Modern Math whose cluster text is `text`, page 1.
fn math_face_glyphs(body: &str, text: &str) -> Vec<(u16, f64, f64, f64)> {
    use flashtex_compiler::parser::SourceDocument;
    use flashtex_render_pipeline::display::RunRole;
    use flashtex_render_pipeline::ids::GlyphId;
    use flashtex_render_pipeline::{render, FontSet, RenderOptions};
    let fonts = FontSet::with_default_dirs(&[]);
    let text_doc = doc(body);
    let r = render(&[SourceDocument { path: "main.tex", text: &text_doc }], "main.tex", 7, "test-project", &fonts, &RenderOptions::default());
    let text_baseline = r.v2.pages[0]
        .items
        .iter()
        .find_map(|item| match item {
            Item::GlyphRun(run) if run.role == RunRole::Text => Some(run.glyphs[0].baseline_y.to_bp()),
            _ => None,
        })
        .expect("a text word");
    let mut out = Vec::new();
    for item in &r.v2.pages[0].items {
        if let Item::GlyphRun(run) = item {
            if run.role != RunRole::Math {
                continue;
            }
            let face = fonts.by_font_id(&run.font_id).expect("resource of a drawn run");
            if !face.name.to_ascii_lowercase().contains("math") {
                continue;
            }
            let size = run.font_size.to_bp();
            for g in &run.glyphs {
                let c = &run.clusters[g.cluster as usize];
                if &run.text[c.text_start_byte as usize..c.text_end_byte as usize] != text {
                    continue;
                }
                let b = face.bounds(GlyphId(g.gid), None);
                let (top, bottom) = (face.pt(i64::from(b.y_max), size), face.pt(-i64::from(b.y_min), size));
                let y = g.baseline_y.to_bp() - text_baseline;
                out.push((g.gid, top + bottom, y - top, y + bottom));
            }
        }
    }
    out
}

/// The cmex chain step math-layout selects (`\Big(` = cmex 0x10, 18 pt, for
/// a `\left(` around a text-style fraction; `\bigg(` = 0x12, 24 pt, in
/// display — pdflatex sets `lmex10` codes 0x10/0x12 there in the oracle
/// PDFs) must be painted with the Latin Modern Math variant of that size.
/// The face lists seven parenthesis sizes against cmex's four, so a
/// same-index pick draws 11.9 pt / 14.4 pt glyphs where TeX sets 18 / 24.
#[test]
fn left_right_paints_the_variant_of_the_selected_cmex_size() {
    if !lm_available() {
        return;
    }
    let inline = math_face_glyphs("Inline $\\left( \\frac{a}{b} \\right)$ text.", "(");
    let display = math_face_glyphs("Display \\[ \\left( \\frac{a}{b} \\right) \\] after.", "(");
    assert_eq!(inline.len(), 1, "{inline:?}");
    assert_eq!(display.len(), 1, "{display:?}");
    let (gid_t, h_t, top_t, bottom_t) = inline[0];
    let (gid_d, h_d, _, _) = display[0];
    // cmex10 is `sfixed` at 10 pt: the \Big box is 18 pt, the \bigg box 24 pt.
    assert!((h_t - 18.0).abs() < 0.3, "inline \\left( draws gid {gid_t}, {h_t:.2} pt tall; TeX's \\Big( box is 18 pt");
    assert!((h_d - 24.0).abs() < 0.3, "display \\left( draws gid {gid_d}, {h_d:.2} pt tall; TeX's \\bigg( box is 24 pt");
    assert_ne!(gid_t, gid_d);
    // Vertical placement: pdflatex sets the \Big( origin 11.557 pt above the
    // text baseline (oracle content stream, 08-delimiters), and the cmex
    // outline hangs from it (0.4 pt above, 17.6 below): ink from 11.96 pt
    // above the baseline to 6.04 pt below. The OpenType variant is centred
    // on the axis relative to its own origin, so painting it at the cmex
    // origin would lift it ~9 pt; the painter re-centres it on the TFM box.
    assert!((top_t + 11.96).abs() < 0.3, "inline \\left( ink top {top_t:.2} pt from the baseline; TeX: -11.96");
    assert!((bottom_t - 6.04).abs() < 0.3, "inline \\left( ink bottom {bottom_t:.2} pt from the baseline; TeX: +6.04");
    // The radical chain lists exactly the cmex sizes: \sqrt over a text-style
    // fraction takes cmex 0x71 (18 pt) and paints the 18 pt sign.
    let sqrt = math_face_glyphs("Inline $\\sqrt{\\frac{a}{b}}$ text.", "\u{221A}");
    assert_eq!(sqrt.len(), 1, "{sqrt:?}");
    assert!((sqrt[0].1 - 18.0).abs() < 0.3, "\\sqrt sign gid {} is {:.2} pt tall; cmex 0x71 is 18 pt", sqrt[0].0, sqrt[0].1);
    // Text-size \sum is a cmex glyph too (0x50, hanging 10 pt below its
    // origin); centred on the axis (3 pt at 12 pt) its ink spans 8 pt above
    // to 2 pt below the baseline, as pdflatex draws lmex10.
    let sum = math_face_glyphs("Inline $\\sum_{i=1}^{n} i$ text.", "\u{2211}");
    assert_eq!(sum.len(), 1, "{sum:?}");
    let (_, _, top_s, bottom_s) = sum[0];
    assert!((top_s + 8.0).abs() < 0.4 && (bottom_s - 2.0).abs() < 0.4, "text \\sum ink {top_s:.2}..{bottom_s:.2} pt from the baseline; TeX: -8.0..+2.0");
}

/// Symbols with no Computer Modern slot (`\mathbb`, `\setminus`,
/// `\Longrightarrow`, `\aleph`) are drawn from Latin Modern Math through
/// `OTF_FALLBACK_FONT`, whose id lies above the `\text` run range: the
/// painter must not look them up as run glyphs (they were silently
/// dropped). The math minus is U+2212, not the text hyphen.
#[test]
fn cm_less_symbols_are_painted_from_latin_modern_math() {
    if !lm_available() {
        eprintln!("skipping: Latin Modern not installed");
        return;
    }
    let r = render_one(&doc("A $\\mathbb{Z}\\aleph$ $\\mathbb{R}\\setminus\\mathbb{Q}$ $x\\Longrightarrow y$ $10 - x$ B"));
    let runs: Vec<(String, u16)> = r.v2.pages[0]
        .items
        .iter()
        .filter_map(|it| match it {
            Item::GlyphRun(run) => Some((run.text.clone(), run.glyphs[0].gid)),
            _ => None,
        })
        .collect();
    let texts: Vec<&str> = runs.iter().map(|(t, _)| t.as_str()).collect();
    assert!(texts.contains(&"ℤℵ"), "\\mathbb{{Z}}\\aleph dropped: {texts:?}");
    assert!(texts.contains(&"ℝ∖ℚ"), "\\mathbb{{R}}\\setminus\\mathbb{{Q}} dropped: {texts:?}");
    assert!(texts.contains(&"x⟹y"), "\\Longrightarrow dropped: {texts:?}");
    // The run text keeps the source's ASCII hyphen; the painted glyph is
    // Latin Modern Math's U+2212 (gid 2615 in the pinned font 6075562b…),
    // not its text hyphen (gid 14).
    let minus = runs.iter().find(|(t, _)| t.starts_with('-')).expect("the minus run");
    assert_eq!(minus.1, 2615, "math minus should paint U+2212: {runs:?}");
}
