//! Theorem-layout gate: word origins and end-of-proof box rules against the
//! pdfLaTeX (MacTeX 2026, pdfTeX 1.40.29) references pinned in
//! `tests/theorem_corpus/refs` (made by that corpus's `oracle.py refs`,
//! oracle tooling only; cargo never runs TeX).
//!
//! The corpus covers the kernel's `\newtheorem` without amsthm, amsthm's
//! `plain`/`definition`/`remark` styles, `\newtheoremstyle` (punctuation,
//! indent, `\newline` head), shared and `[within]` counters,
//! `\numberwithin`, starred environments, notes, `\swapnumbers`, `proof`
//! with its box on the last line, on a full last line and after `\qedhere`
//! in `\[`/`equation*`/lists, theorems starting with a list or holding a
//! display, page breaks, `\label`/`\ref`, headings, two columns and
//! `\parskip`, at 10/11/12pt.
//!
//! For every fixture in `PASSING`: the page count matches, every reference
//! word has a candidate glyph origin within 0.5 bp in x and y, and every
//! reference rule (the `\openbox` strokes) has a candidate rule within
//! 0.5 bp in position and size. `oracle.py check` measures all fixtures,
//! including the ones listed in `PENDING` with the reason they do not match
//! yet.

mod common;

use flashtex_compiler::json::{self, Value};
use flashtex_render_pipeline::display::Item;

const TOL: f64 = 0.5;

const PASSING: &[&str] = &[
    "01-kernel-10pt",
    "01-kernel-11pt",
    "01-kernel-12pt",
    "02-kernel-within-section-11pt",
    "03-plain-10pt",
    "03-plain-11pt",
    "03-plain-12pt",
    "04-definition-10pt",
    "05-remark-11pt",
    "06-styles-mixed-12pt",
    "07-newtheoremstyle-10pt",
    "07-newtheoremstyle-indent-11pt",
    "07-newtheoremstyle-newline-12pt",
    "08-shared-within-10pt",
    "09-starred-11pt",
    "10-note-long-12pt",
    "11-numberwithin-10pt",
    "12-swapnumbers-10pt",
    "13-proof-10pt",
    "13-proof-11pt",
    "13-proof-12pt",
    "14-proof-fullline-10pt",
    "15-proof-qedhere-displaymath-10pt",
    "15-proof-qedhere-equation-11pt",
    "17-proof-name-12pt",
    "18-proof-multipar-10pt",
    "22-label-ref-10pt",
    "23-after-heading-11pt",
    "24-theorem-then-proof-12pt",
    "26-theorem-multipar-11pt",
    "27-kernel-within-12pt",
    "28-twocolumn-10pt",
    "29-proof-in-theorem-text-qed-10pt",
    "30-remark-numbered-12pt",
    "31-proof-noindent-after-11pt",
    "32-theorem-in-parskip-10pt",
];

/// Fixtures `oracle.py check` still reports as failing, and why.
#[allow(dead_code)]
const PENDING: &[(&str, &str)] = &[
    ("16-proof-qedhere-enumerate-12pt", "list lane (#143): list indents at 12pt use 12pt, not the font's quad (0.62 bp)"),
    ("16-proof-qedhere-itemize-10pt", "list lane (#143): the bullet label is 2.77 bp left (cmsy widths)"),
    ("19-theorem-itemize-after-head-11pt", "list lane (#143): the bullet label is 3.06 bp left (cmsy widths)"),
    ("19-theorem-enumerate-10pt", "theorem opening with a list: amsthm's head joins the first item's label (`\\@donoparitem`), not implemented"),
    ("20-theorem-display-11pt", "display lane: the same display is 1 bp off without a theorem"),
    ("21-pagebreak-10pt", "page builder: see oracle.py check"),
    ("25-qedsymbol-10pt", "vendored compiler: a renewed `\\qedsymbol` (math) is emitted as U+220E"),
];

fn num(v: &Value) -> f64 {
    match v {
        Value::Num(n) => *n,
        _ => f64::NAN,
    }
}

#[test]
fn theorem_fixtures_match_pdflatex() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/theorem_corpus");
    if !common::lm_available() {
        eprintln!("SKIP theorem_oracle: Latin Modern fonts not installed");
        return;
    }
    let mut failures = Vec::new();
    for name in PASSING {
        let tex = std::fs::read_to_string(format!("{dir}/fixtures/{name}.tex")).unwrap();
        let reference = json::parse(&std::fs::read_to_string(format!("{dir}/refs/{name}.json")).unwrap()).unwrap();
        let ref_pages = reference.get("pages").and_then(|p| p.as_arr()).unwrap();
        let ref_rules = reference.get("rules").and_then(|r| r.as_arr());
        let rendered = common::render_one(&tex);
        let pages = &rendered.v2.pages;
        if pages.len() != ref_pages.len() {
            failures.push(format!("{name}: {} pages vs reference {}", pages.len(), ref_pages.len()));
            continue;
        }
        for (pi, (page, words)) in pages.iter().zip(ref_pages).enumerate() {
            let mut glyphs = Vec::new();
            let mut rules = Vec::new();
            for item in &page.items {
                match item {
                    Item::GlyphRun(run) => glyphs.extend(run.glyphs.iter().map(|g| (g.origin_x.to_bp(), g.baseline_y.to_bp()))),
                    Item::Rule(r) => rules.push([r.x.to_bp(), r.top.to_bp(), r.width.to_bp(), r.height.to_bp()]),
                    _ => {}
                }
            }
            for w in words.as_arr().unwrap() {
                let (x, y) = (num(w.get("x").unwrap()), num(w.get("y_top").unwrap()));
                if !glyphs.iter().any(|(gx, gy)| (gx - x).abs() <= TOL && (gy - y).abs() <= TOL) {
                    failures.push(format!("{name}: page {}: no glyph at word {:?} ({x:.3}, {y:.3})", page.number, w.get("text").and_then(|t| t.as_str()).unwrap_or("")));
                }
            }
            let expected = ref_rules.and_then(|r| r.get(pi)).and_then(|r| r.as_arr()).map_or(&[][..], |r| r.as_slice());
            if expected.len() != rules.len() {
                failures.push(format!("{name}: page {}: {} rules vs reference {}", page.number, rules.len(), expected.len()));
            }
            for rule in expected {
                let want: Vec<f64> = rule.as_arr().unwrap().iter().map(num).collect();
                if !rules.iter().any(|got| got.iter().zip(&want).all(|(g, w)| (g - w).abs() <= TOL)) {
                    failures.push(format!("{name}: page {}: no rule at {want:?}", page.number));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{} mismatches:\n{}", failures.len(), failures.join("\n"));
}
