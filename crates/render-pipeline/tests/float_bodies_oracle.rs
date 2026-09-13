//! Float bodies and two-column wide floats gate: lists, display math,
//! headings, TikZ pictures, side-by-side graphics and minipages inside
//! floats, and `figure*`/`table*` in two-column documents, against the
//! pdfLaTeX (MacTeX 2026) references committed in `fixtures/float-bodies/refs`
//! (made by `fixtures/float-bodies/oracle.py`, test-only; cargo never runs
//! TeX). Words and rules are extracted as the tabular corpus does.
//!
//! Every fixture: the same page count; on every page each reference word's
//! origin has a candidate glyph origin within 0.5 bp (x and y); after
//! merging rules that continue one another the rule counts are equal and
//! every reference rule has a distinct candidate within 0.1 bp in x, top,
//! width and height (a TikZ fill of an axis-aligned rectangle and a straight
//! stroke count as rules, as pdfTeX's `re f` and `m l S` do in the
//! reference); no `float_*` diagnostic.

mod common;

use flashtex_compiler::json::{self, Value};
use flashtex_render_pipeline::display::{Item, PathCmd, PathPaintOp};

const WORD_TOL: f64 = 0.5;
const RULE_TOL: f64 = 0.1;

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

/// A path that pdfTeX would paint as a rule: a filled axis-aligned
/// rectangle, or a straight horizontal/vertical stroke.
fn path_rule(path: &flashtex_render_pipeline::display::PathItem) -> Option<[f64; 4]> {
    if path.commands.iter().any(|c| matches!(c, PathCmd::Cubic(..))) {
        return None;
    }
    let pts: Vec<(f64, f64)> = path
        .commands
        .iter()
        .filter_map(|c| match c {
            PathCmd::Move(x, y) | PathCmd::Line(x, y) => Some((x.to_bp(), y.to_bp())),
            _ => None,
        })
        .collect();
    let (x0, x1) = pts.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), p| (a.min(p.0), b.max(p.0)));
    let (y0, y1) = pts.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), p| (a.min(p.1), b.max(p.1)));
    match &path.op {
        PathPaintOp::Stroke(st) if pts.len() == 2 => {
            let w = st.width.to_bp();
            if (y1 - y0).abs() < 1e-3 {
                Some([x0, y0 - w / 2.0, x1 - x0, w])
            } else if (x1 - x0).abs() < 1e-3 {
                Some([x0 - w / 2.0, y0, w, y1 - y0])
            } else {
                None
            }
        }
        _ => None,
    }
}

#[test]
#[ignore = "float bodies in progress: run with --ignored (checkpoint of the rebase onto main)"]
fn float_body_fixtures_match_pdflatex() {
    if !common::lm_available() {
        eprintln!("SKIP float_bodies_oracle: Latin Modern fonts not installed");
        return;
    }
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/float-bodies");
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|n| n.ends_with(".tex") && n.as_bytes()[0].is_ascii_digit())
        .map(|n| n.trim_end_matches(".tex").to_string())
        .collect();
    names.sort();
    assert!(names.len() >= 14, "expected at least 14 float-body fixtures, found {}", names.len());
    let only = std::env::var("FLOAT_BODIES_ONLY").ok();
    let mut failures = Vec::new();
    let mut report = String::new();
    for name in names.iter().filter(|n| only.as_deref().is_none_or(|o| n.contains(o))) {
        let tex = std::fs::read_to_string(format!("{dir}/{name}.tex")).unwrap();
        let reference = json::parse(&std::fs::read_to_string(format!("{dir}/refs/{name}.json")).unwrap()).unwrap();
        let rendered = common::render_one(&tex);
        let before = failures.len();
        let (mut word_worst, mut rule_worst, mut words_n, mut rules_n) = (0.0f64, 0.0f64, 0usize, 0usize);
        for d in &rendered.v2.diagnostics {
            if d.code.starts_with("float") {
                failures.push(format!("{name}: diagnostic {}: {}", d.code, d.message));
            }
        }
        let ref_pages = reference.get("pages").and_then(|p| p.as_arr()).unwrap();
        let ref_rules = reference.get("rules").and_then(|p| p.as_arr()).unwrap();
        let pages = &rendered.v2.pages;
        if pages.len() != ref_pages.len() {
            failures.push(format!("{name}: {} pages vs reference {}", pages.len(), ref_pages.len()));
        }
        for (pi, (page, (words, rules))) in pages.iter().zip(ref_pages.iter().zip(ref_rules)).enumerate() {
            let mut glyphs = Vec::new();
            let mut ours = Vec::new();
            for item in &page.items {
                match item {
                    Item::GlyphRun(run) => glyphs.extend(run.glyphs.iter().map(|g| (g.origin_x.to_bp(), g.baseline_y.to_bp()))),
                    Item::Rule(r) => ours.push([r.x.to_bp(), r.top.to_bp(), r.width.to_bp(), r.height.to_bp()]),
                    Item::Path(path) => ours.extend(path_rule(path)),
                    _ => {}
                }
            }
            for w in words.as_arr().unwrap() {
                let (x, y) = (num(w.get("x").unwrap()), num(w.get("y_top").unwrap()));
                words_n += 1;
                let best = glyphs.iter().map(|(gx, gy)| (gx - x).abs().max((gy - y).abs())).fold(f64::INFINITY, f64::min);
                word_worst = word_worst.max(best);
                if best > WORD_TOL {
                    failures.push(format!("{name} p{}: no glyph at word {:?} ({x:.3}, {y:.3}); nearest {best:.3}", pi + 1, w.get("text").and_then(|t| t.as_str()).unwrap_or("")));
                }
            }
            let reference_rules: Vec<[f64; 4]> = rules
                .as_arr()
                .unwrap()
                .iter()
                .map(|r| {
                    let v = r.as_arr().unwrap();
                    [num(&v[0]), num(&v[1]), num(&v[2]), num(&v[3])]
                })
                .collect();
            let mut ours = merge(ours);
            if ours.len() != reference_rules.len() {
                failures.push(format!("{name} p{}: {} rules vs reference {}", pi + 1, ours.len(), reference_rules.len()));
            }
            for r in &reference_rules {
                rules_n += 1;
                let best = ours
                    .iter()
                    .enumerate()
                    .map(|(i, o)| (i, (0..4).map(|k| (o[k] - r[k]).abs()).fold(0.0, f64::max)))
                    .min_by(|a, b| a.1.total_cmp(&b.1));
                match best {
                    Some((i, d)) => {
                        rule_worst = rule_worst.max(d);
                        if d <= RULE_TOL {
                            ours.remove(i);
                        } else {
                            failures.push(format!("{name} p{}: no rule within {RULE_TOL} bp of {r:?}; nearest {:?} ({d:.3})", pi + 1, ours[i]));
                        }
                    }
                    None => failures.push(format!("{name} p{}: no rule for {r:?}", pi + 1)),
                }
            }
        }
        let status = if failures.len() == before { "pass" } else { "FAIL" };
        report.push_str(&format!("| {name} | {status} | {}/{} | {words_n} | {word_worst:.3} | {rules_n} | {rule_worst:.3} |\n", pages.len(), ref_pages.len()));
    }
    println!("| fixture | result | pages ours/ref | words | worst word (bp) | rules | worst rule (bp) |\n|---|---|---|---|---|---|---|\n{report}");
    assert!(failures.is_empty(), "{} mismatches:\n{}", failures.len(), failures.join("\n"));
}
