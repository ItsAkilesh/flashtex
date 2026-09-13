//! Math font sizes and the spacing of commands the compiler's atoms do not
//! carry, against pdfLaTeX (MacTeX 2026, oracle only).
//!
//! Reference positions come from math-layout's `tools/size_oracle.py` run
//! (`tests/size_oracle/expected.txt` there): origins in bp of every glyph of
//! one inline formula in an article, relative to the formula's first glyph.
//! The pipeline draws Latin Modern outlines with the CM/LM TFM widths, so the
//! origins must agree with pdfTeX's.

mod common;

use common::*;
use flashtex_render_pipeline::display::{Item, Tick};

const TOLERANCE_BP: f64 = 0.3;

/// Glyph origins `(x, y)` in bp (y down) of the first line of page 1,
/// sorted by x, relative to the leftmost glyph.
fn formula_origins(src: &str) -> Vec<(f64, f64)> {
    let r = render_one(src);
    let mut all: Vec<(f64, f64)> = Vec::new();
    for item in &r.v2.pages[0].items {
        if let Item::GlyphRun(run) = item {
            for g in &run.glyphs {
                all.push((g.origin_x.to_bp(), g.baseline_y.to_bp()));
            }
        }
    }
    let top = all.iter().map(|g| g.1).fold(f64::INFINITY, f64::min);
    // The formula's line: baselines within 10bp of the highest glyph (scripts
    // sit above the baseline), which excludes the page number.
    let mut line: Vec<(f64, f64)> = all.into_iter().filter(|g| g.1 < top + 10.0).collect();
    line.sort_by(|a, b| a.0.total_cmp(&b.0));
    let base = *line.iter().max_by(|a, b| a.1.total_cmp(&b.1)).expect("glyphs");
    let (x0, _) = line[0];
    line.iter().map(|g| (g.0 - x0, base.1 - g.1)).collect()
}

/// Asserts relative x origins (and y raises where given) match pdfTeX's.
fn assert_origins(src: &str, want_x: &[f64], want_raise: &[f64]) {
    let got = formula_origins(src);
    assert_eq!(got.len(), want_x.len(), "{src}: glyphs {got:?}");
    for (i, ((x, raise), wx)) in got.iter().zip(want_x).enumerate() {
        assert!((x - wx).abs() <= TOLERANCE_BP, "{src}: glyph {i} x {x:.3}bp, pdfTeX {wx:.3}bp ({got:?})");
        if let Some(wr) = want_raise.get(i) {
            assert!((raise - wr).abs() <= TOLERANCE_BP, "{src}: glyph {i} raised {raise:.3}bp, pdfTeX {wr:.3}bp ({got:?})");
        }
    }
}

fn article(options: &str, packages: &str, body: &str) -> String {
    format!("\\documentclass[{options}]{{article}}\n{packages}\n\\begin{{document}}\n\\noindent {body}\n\\end{{document}}\n")
}

#[test]
fn amsmath_dots_before_a_comma_is_ldots_with_inner_spacing() {
    // size_oracle `dotsc-10a`: a , . . . , b
    assert_origins(
        &article("10pt", "\\usepackage{amsmath}", "$a,\\dots,b$"),
        &[0.0, 5.266, 9.698, 14.119, 18.550, 22.982, 27.403],
        &[],
    );
}

#[test]
fn kernel_ldots_is_mathellipsis() {
    // size_oracle `ldots-10`: ( 1 , . . . , n )
    assert_origins(
        &article("10pt", "", "$(1,\\ldots,n)$"),
        &[0.0, 3.874, 8.856, 13.287, 17.709, 22.140, 26.571, 30.993, 36.972],
        &[],
    );
}

#[test]
fn amsmath_bmod_has_five_mu_each_side() {
    // size_oracle `bmod-10a`: a m o d b
    assert_origins(&article("10pt", "\\usepackage{amsmath}", "$a\\bmod b$"), &[0.0, 8.036, 16.338, 21.598, 29.893], &[]);
}

#[test]
fn prime_is_cmsy_prime_at_script_size() {
    // size_oracle `prime-f-10`: f ′ ( x )
    assert_origins(&article("10pt", "", "$f'(x)$"), &[0.0, 5.950, 8.745, 12.619, 18.313], &[0.0, 3.616, 0.0, 0.0, 0.0]);
}

#[test]
fn one_letter_mathrm_takes_character_script_placement() {
    // size_oracle `k-inv-10`: K − 1, the exponent 3.616bp up (sup2), not
    // the box rule's height − sup_drop.
    assert_origins(&article("10pt", "", "$\\mathrm{K}^{-1}$"), &[0.0, 7.749, 13.976], &[0.0, 3.616, 3.616]);
}

#[test]
fn epsilon_and_varepsilon_are_different_cmmi_slots() {
    // size_oracle `epsilon-10`: cmmi "0F (4.047pt) + cmmi "22 (4.66pt).
    assert_origins(&article("10pt", "", "$\\epsilon+\\varepsilon$"), &[0.0, 6.256, 16.216], &[]);
}

#[test]
fn footnote_math_uses_the_footnotesize_math_fonts() {
    // An 11pt class's `\footnotesize` is 9pt: `\DeclareMathSizes{9}{9}{6}{5}`.
    let r = render_one(&article("11pt", "\\usepackage{amsmath}", "Text.\\footnote{$\\mathrm{K}^{-1}$}"));
    let sizes: Vec<(String, Tick)> = r.v2.pages[0]
        .items
        .iter()
        .filter_map(|item| match item {
            Item::GlyphRun(run) => Some((run.text.clone(), run.font_size)),
            _ => None,
        })
        .collect();
    assert!(sizes.contains(&("K".to_string(), Tick::from_tex_pt(9.0))), "{sizes:?}");
    // The exponent's `1` (cmr6) at the scriptsize of 9pt math.
    assert!(sizes.contains(&("1".to_string(), Tick::from_tex_pt(6.0))), "{sizes:?}");
    // Body math stays at the class's 10.95pt.
    let body = render_one(&article("11pt", "\\usepackage{amsmath}", "$\\mathrm{K}^{-1}$"));
    let body_k: Vec<Tick> = body.v2.pages[0]
        .items
        .iter()
        .filter_map(|item| match item {
            Item::GlyphRun(run) if run.text == "K" => Some(run.font_size),
            _ => None,
        })
        .collect();
    assert_eq!(body_k, vec![Tick::from_tex_pt(10.95)]);
}
