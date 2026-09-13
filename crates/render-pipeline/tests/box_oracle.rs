//! LaTeX box commands (`\mbox`, `\makebox`, `\fbox`, `\framebox`,
//! `\parbox`, `minipage`, `\raisebox`, phantoms, `\smash`, laps, `\strut`,
//! saved boxes, `\settowidth` & co.) against pdfLaTeX (TeX Live 2026).
//!
//! References are pinned in `fixtures/boxes/refs` by
//! `tools/box-oracle/oracle.py refs` (test-only; cargo never runs TeX). For
//! every fixture in `PASSING`: one page; every reference word's origin has a
//! candidate glyph origin within 0.5 bp (x and y); after merging rules that
//! continue one another, the rule counts are equal and every reference rule
//! has a distinct candidate rule within 0.1 bp in x, top, width and height.
//! Skips (loudly) when Latin Modern is not installed.

mod common;

use flashtex_compiler::json::{self, Value};
use flashtex_render_pipeline::display::Item;

const WORD_TOL: f64 = 0.5;
const RULE_TOL: f64 = 0.1;

/// Fixtures that match pdfLaTeX today; see `KNOWN_DIFFERENCES` for the rest.
const PASSING: &[&str] = &[
    "01-mbox", "02-makebox-l", "03-makebox-c", "04-makebox-r", "05-makebox-s", "06-makebox-width-factor",
    "07-makebox-zero", "08-fbox", "09-fbox-sep-rule", "10-framebox-r", "11-framebox-l-c", "12-fbox-math",
    "13-nested", "14-raisebox", "15-raisebox-height-depth", "16-parbox-t", "17-parbox-c", "18-parbox-b",
    "19-parbox-height-inner",
    "20-minipage-t", "21-minipage-side-by-side", "22-minipage-paragraphs", "24-phantom", "25-vphantom-line",
    "26-smash", "27-llap-rlap", "28-strut", "29-sbox-usebox", "30-savebox-lrbox", "31-settowidth-hspace",
    "32-settoheight-depth", "33-hss", "34-makebox-hfill", "35-parbox-center-env", "36-11pt-fbox-parbox",
    "37-10pt-raisebox-framebox",
];

/// Fixtures that do not match yet, with the reason.
#[allow(dead_code)]
const KNOWN_DIFFERENCES: &[(&str, &str)] = &[
    ("23-minipage-footnote", "minipage footnotes (`\\@mpfootins`) are not implemented; the note is set inline"),
];

fn num(v: &Value) -> f64 {
    match v {
        Value::Num(n) => *n,
        _ => f64::NAN,
    }
}

/// Merges rules (x, top, width, height) that continue one another.
fn merge(mut rules: Vec<[f64; 4]>) -> Vec<[f64; 4]> {
    loop {
        let mut joined = None;
        'outer: for i in 0..rules.len() {
            for j in 0..rules.len() {
                let (a, b) = (rules[i], rules[j]);
                if i == j {
                    continue;
                }
                let vertical = (a[0] - b[0]).abs() < 0.01 && (a[2] - b[2]).abs() < 0.01 && (a[1] + a[3] - b[1]).abs() < 0.01;
                let horizontal = (a[1] - b[1]).abs() < 0.01 && (a[3] - b[3]).abs() < 0.01 && (a[0] + a[2] - b[0]).abs() < 0.01;
                if vertical || horizontal {
                    joined = Some((i, j, vertical));
                    break 'outer;
                }
            }
        }
        let Some((i, j, vertical)) = joined else { return rules };
        let b = rules[j];
        if vertical {
            rules[i][3] = b[1] + b[3] - rules[i][1];
        } else {
            rules[i][2] = b[0] + b[2] - rules[i][0];
        }
        rules.remove(j);
    }
}

#[test]
fn box_fixtures_match_pdflatex() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/boxes");
    if !common::lm_available() {
        eprintln!("SKIP box_oracle: Latin Modern fonts not installed");
        return;
    }
    let mut failures = Vec::new();
    for name in PASSING {
        let tex = std::fs::read_to_string(format!("{dir}/{name}.tex")).unwrap();
        let reference = json::parse(&std::fs::read_to_string(format!("{dir}/refs/{name}.json")).unwrap()).unwrap();
        let rendered = common::render_one(&tex);
        let pages = &rendered.v2.pages;
        if pages.len() != 1 {
            failures.push(format!("{name}: {} pages", pages.len()));
            continue;
        }
        let mut glyphs = Vec::new();
        let mut rules = Vec::new();
        for item in &pages[0].items {
            match item {
                Item::GlyphRun(run) => glyphs.extend(run.glyphs.iter().map(|g| (g.origin_x.to_bp(), g.baseline_y.to_bp()))),
                Item::Rule(r) => rules.push([r.x.to_bp(), r.top.to_bp(), r.width.to_bp(), r.height.to_bp()]),
                _ => {}
            }
        }
        let words = reference.get("pages").and_then(|p| p.as_arr()).and_then(|p| p.first()).and_then(|p| p.as_arr()).unwrap();
        for w in words {
            let (x, y) = (num(w.get("x").unwrap()), num(w.get("y_top").unwrap()));
            if !glyphs.iter().any(|(gx, gy)| (gx - x).abs() <= WORD_TOL && (gy - y).abs() <= WORD_TOL) {
                failures.push(format!("{name}: no glyph at word {:?} ({x:.3}, {y:.3})", w.get("text").and_then(|t| t.as_str()).unwrap_or("")));
            }
        }
        let ref_rules: Vec<[f64; 4]> = reference
            .get("rules")
            .and_then(|r| r.as_arr())
            .and_then(|r| r.first())
            .and_then(|r| r.as_arr())
            .unwrap()
            .iter()
            .map(|r| {
                let v = r.as_arr().unwrap();
                [num(&v[0]), num(&v[1]), num(&v[2]), num(&v[3])]
            })
            .collect();
        let mut ours = merge(rules);
        if ours.len() != ref_rules.len() {
            failures.push(format!("{name}: {} rules vs reference {}", ours.len(), ref_rules.len()));
        }
        for r in &ref_rules {
            let best = ours
                .iter()
                .enumerate()
                .map(|(i, o)| (i, (0..4).map(|k| (o[k] - r[k]).abs()).fold(0.0, f64::max)))
                .min_by(|a, b| a.1.total_cmp(&b.1));
            match best {
                Some((i, d)) if d <= RULE_TOL => {
                    ours.remove(i);
                }
                _ => failures.push(format!("{name}: no rule within {RULE_TOL} bp of {r:?}")),
            }
        }
    }
    assert!(failures.is_empty(), "{} mismatches:\n{}", failures.len(), failures.join("\n"));
}

#[test]
fn box_commands_emit_no_unsupported_diagnostics() {
    let src = "\\documentclass{article}\n\\newsavebox{\\keep}\\newlength{\\w}\n\\begin{document}\nA \\mbox{b} \\fbox{c} \\makebox[2cm][r]{d} \\framebox[1cm]{e} \\raisebox{1pt}[2pt][1pt]{f} \\parbox[t]{2cm}{g h} \\begin{minipage}[b]{3cm}i\\end{minipage} \\phantom{j}\\hphantom{k}\\vphantom{l}\\smash{m}\\llap{n}\\rlap{o}\\strut \\sbox{\\keep}{p}\\usebox{\\keep}\\settowidth{\\w}{q}\\hspace{\\w}r\n\\end{document}\n";
    let rendered = common::render_one(src);
    let bad: Vec<_> = rendered
        .v2
        .diagnostics
        .iter()
        .filter(|d| d.severity == flashtex_render_pipeline::display::Severity::Error || d.code.contains("unsupported") || d.code == "box_limitation")
        .map(|d| format!("{} {}", d.code, d.message))
        .collect();
    assert!(bad.is_empty(), "{bad:?}");
}
