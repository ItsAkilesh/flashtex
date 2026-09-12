//! The compiler's control-word symbols (pin `49e6eb43`) through TeX's
//! metrics and Latin Modern Math: composites (`\neq` = `\not=`), the extra
//! cmsy slots (`\perp`), `\cdot`'s class, `\left`/`\right` fences re-derived
//! from the source, and the typed limitation for `\angle`.

mod common;

use common::*;
use flashtex_render_pipeline::display::Item;
use flashtex_render_pipeline::typeset::{convert_math, fence_before, Fence};
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
fn every_compiler_symbol_typesets_without_a_missing_glyph_except_angle() {
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
    assert_eq!(limitations.len(), 1, "{limitations:?}");
    assert!(limitations[0].contains('\u{2220}'), "only \\angle (a constructed macro in fontmath.ltx, not a glyph) is unmatched: {limitations:?}");
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
