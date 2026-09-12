//! Emits the positioned runs of the oracle-comparison formulas as JSON for
//! `tools/oracle_compare.py`: stage 1 (`docs/oracle/compare.tex`, formulas
//! A–C) by default, stage 2 (`compare2.tex`, D–G) with the argument `2`.
//! Units are TeX points with a top-left origin at (0, 0) per formula.

use flashtex_math_layout::{
    Atom, CmMathMetrics, MathFontMetrics, MathList, Style, fixtures, layout_with_report,
    positioned_runs,
};

fn formulas(stage: u32) -> Vec<(&'static str, MathList)> {
    if stage == 2 {
        // D: \left\{ tall stack \right\}  (extensible brace)
        // E: \sqrt[3]{\frac{a}{b}}
        // F: \lim_{x\to 0}\frac{\sin x}{x}
        // G: \sqrt{ tall stack }  (extensible radical over the brace stack)
        let g = MathList::from(Atom::sqrt(fixtures::tall_braces()));
        return vec![
            ("D", fixtures::tall_braces()),
            ("E", fixtures::cube_root_frac()),
            ("F", fixtures::lim_sin_x_over_x()),
            ("G", g),
        ];
    }
    // A: \frac{\frac{a}{b}}{c}=1
    let mut a = fixtures::stacked_fraction();
    a.atoms.push(Atom::symbol('='));
    a.atoms.push(Atom::symbol('1'));
    // B: \sqrt{x}+\left(\frac{a}{b}\right)
    let mut b = fixtures::sqrt_x();
    b.atoms.push(Atom::symbol('+'));
    b.atoms.extend(fixtures::left_right_frac().atoms);
    // C: \sum_{i=1}^{n}x_i^2
    let mut c = fixtures::sum_limits();
    c.atoms.extend(fixtures::x_sub_i_sup_2().atoms);
    vec![("A", a), ("B", b), ("C", c)]
}

fn main() {
    let stage: u32 = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(1);
    let m = CmMathMetrics::latex_10pt();
    let mut out = String::from("[");
    for (i, (name, list)) in formulas(stage).into_iter().enumerate() {
        let l = layout_with_report(&list, Style::DISPLAY, &m);
        let runs = positioned_runs(&l.root, (0.0, 0.0));
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!(
            "{{\"name\":\"{name}\",\"height\":{},\"depth\":{},\"width\":{},\"glyphs\":[",
            l.root.height, l.root.depth, l.root.width
        ));
        for (j, g) in runs.glyphs.iter().enumerate() {
            if j > 0 {
                out.push(',');
            }
            out.push_str(&format!(
                "{{\"font\":\"{}\",\"gid\":{},\"ch\":\"{}\",\"x\":{},\"baseline\":{},\"size\":{}}}",
                m.font_name(g.font_id),
                g.gid,
                if g.ch == '"' || g.ch == '\\' {
                    format!("\\{}", g.ch)
                } else {
                    g.ch.to_string()
                },
                g.x,
                g.baseline_y,
                g.size
            ));
        }
        out.push_str("],\"rules\":[");
        for (j, r) in runs.rules.iter().enumerate() {
            if j > 0 {
                out.push(',');
            }
            out.push_str(&format!(
                "{{\"x\":{},\"y\":{},\"w\":{},\"h\":{}}}",
                r.x, r.y, r.w, r.h
            ));
        }
        out.push_str("],\"limitations\":[");
        for (j, lim) in l.limitations.iter().enumerate() {
            if j > 0 {
                out.push(',');
            }
            out.push_str(&format!("\"{lim:?}\""));
        }
        out.push_str("]}");
    }
    out.push(']');
    println!("{out}");
}
