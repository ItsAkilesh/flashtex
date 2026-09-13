//! Verbatim/listings gate: glyph origins and rules against the pdfLaTeX
//! (MacTeX 2026) references in `fixtures/verbatim/reference/*.json` (made by
//! `fixtures/verbatim/oracle.py`, test-only; cargo never runs TeX).
//!
//! For every fixture in `PASSING`: the page count matches; on every page the
//! glyph counts are equal and every reference glyph origin has a distinct
//! candidate glyph origin within 0.1 bp in x and y; the rule counts (after
//! merging rules that continue one another) are equal and every reference
//! rule has a candidate within 0.1 bp in x, top, width and height.
//!
//! `VERBATIM_ORACLE_REPORT=1` prints the worst deltas of every fixture,
//! passing or not; `VERBATIM_ORACLE_DUMP=<name>` also prints that fixture's
//! candidate glyphs and rules grouped by baseline.

mod common;

use flashtex_compiler::json::{self, Value};
use flashtex_compiler::parser::SourceDocument;
use flashtex_render_pipeline::display::Item;
use flashtex_render_pipeline::{render, FontSet, RenderOptions};

const TOL: f64 = 0.1;

/// Fixtures that match pdfLaTeX today. `25-lst-java-roman` does not: a
/// listing in the OT1 roman face sets `"` as cmr's `\char34` (a closing
/// double quote) and the visible string space as cmr's slot 32, which the
/// T1 text metrics the pipeline shapes roman text with do not have.
const PASSING: &[&str] = &[
    "01-verbatim-basic", "02-verbatim-ligatures", "03-verbatim-tabs", "04-verbatim-blank-lines", "05-verbatim-star",
    "06-verb-delimiters", "07-verbatim-t1", "08-verbatim-lmodern", "09-verbatim-vmode", "10-verbatim-itemize",
    "11-verbatim-11pt", "12-verbatim-12pt", "13-verbatim-long-line", "14-verbatim-pagebreak", "15-verbatim-small",
    "16-lst-default", "17-lst-tt-fixed", "18-lst-tt-flexible", "19-lst-fullflexible", "20-lst-numbers",
    "21-lst-frame-single", "22-lst-frame-lines-numbers", "23-lst-c-keywords", "24-lst-python-keywords",
    "26-lst-breaklines", "27-lst-showstringspaces", "29-lst-tabs-gobble", "31-lst-t1-lmodern-bold",
    "32-verbatim-microtype",
];

/// Fixtures that also need the compiler to lex `\lstinline` as one verbatim
/// token and to set no text for `\lstset` (crates/compiler, branch
/// agent/kabir-claude/verbatim-fidelity-compiler); they are gated once
/// `vendor/compiler` carries that.
const NEEDS_COMPILER: &[&str] = &["28-lstinline", "30-lst-lstset-margin"];

/// Whether the vendored compiler lexes `\lstinline` (see [`NEEDS_COMPILER`]).
fn compiler_lexes_lstinline() -> bool {
    use flashtex_compiler::parser::{parse, Block, Inline};
    parse("\\lstinline|x|").blocks.iter().any(|b| matches!(b, Block::Paragraph(inlines) if inlines.iter().any(|i| matches!(i, Inline::Verbatim { .. }))))
}

fn num(v: &Value) -> f64 {
    match v {
        Value::Num(n) => *n,
        _ => f64::NAN,
    }
}

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

/// One fixture's comparison: failures and the worst glyph/rule delta.
fn compare(name: &str, dir: &str, fonts: &FontSet) -> (Vec<String>, f64) {
    let tex = std::fs::read_to_string(format!("{dir}/{name}.tex")).unwrap();
    let reference = json::parse(&std::fs::read_to_string(format!("{dir}/reference/{name}.json")).unwrap()).unwrap();
    let options = RenderOptions { project_root: Some(dir.into()), ..RenderOptions::default() };
    let docs = [SourceDocument { path: "main.tex", text: &tex }];
    let rendered = render(&docs, "main.tex", 1, "verbatim", fonts, &options);
    let mut failures = Vec::new();
    let mut worst: f64 = 0.0;
    let pages = &rendered.v2.pages;
    if std::env::var("VERBATIM_ORACLE_DUMP").is_ok_and(|d| d == name) {
        for d in &rendered.v2.diagnostics {
            eprintln!("diagnostic {}: {}", d.code, d.message);
        }
        for page in pages {
            let mut lines: std::collections::BTreeMap<(i64, String), Vec<(f64, String)>> = std::collections::BTreeMap::new();
            for item in &page.items {
                match item {
                    Item::GlyphRun(run) => {
                        for g in &run.glyphs {
                            let text = run
                                .clusters
                                .get(g.cluster as usize)
                                .and_then(|c| run.text.get(c.text_start_byte..c.text_end_byte))
                                .unwrap_or("?")
                                .to_string();
                            lines.entry(((g.baseline_y.to_bp() * 1000.0).round() as i64, run.font_id.chars().take(8).collect())).or_default().push((g.origin_x.to_bp(), text));
                        }
                    }
                    Item::Rule(r) => eprintln!("p{} rule [{:.3}, {:.3}, {:.3}, {:.3}]", page.number, r.x.to_bp(), r.top.to_bp(), r.width.to_bp(), r.height.to_bp()),
                    _ => {}
                }
            }
            for ((y, font), mut glyphs) in lines {
                glyphs.sort_by(|a, b| a.0.total_cmp(&b.0));
                let text: String = glyphs.iter().map(|g| g.1.as_str()).collect();
                eprintln!("p{} y={:8.3} x0={:8.3} {font}: {text:?}", page.number, y as f64 / 1000.0, glyphs[0].0);
            }
        }
    }
    let ref_pages = reference.get("pages").map(num).unwrap_or(0.0) as usize;
    if pages.len() != ref_pages {
        failures.push(format!("{name}: {} pages vs reference {ref_pages}", pages.len()));
    }
    let ref_glyphs = reference.get("glyphs").and_then(|g| g.as_arr()).unwrap();
    let ref_rules = reference.get("rules").and_then(|g| g.as_arr()).unwrap();
    for (index, page) in pages.iter().enumerate() {
        let number = index + 1;
        let mut ours: Vec<(f64, f64)> = Vec::new();
        let mut rules = Vec::new();
        for item in &page.items {
            match item {
                Item::GlyphRun(run) => ours.extend(run.glyphs.iter().map(|g| (g.origin_x.to_bp(), g.baseline_y.to_bp()))),
                Item::Rule(r) => rules.push([r.x.to_bp(), r.top.to_bp(), r.width.to_bp(), r.height.to_bp()]),
                _ => {}
            }
        }
        let theirs: Vec<(f64, f64, String)> = ref_glyphs
            .iter()
            .filter_map(|g| g.as_arr())
            .filter(|g| num(&g[0]) as usize == number)
            .map(|g| (num(&g[1]), num(&g[2]), g[3].as_str().unwrap_or("").to_string()))
            .collect();
        if ours.len() != theirs.len() {
            failures.push(format!("{name} p{number}: {} glyphs vs reference {}", ours.len(), theirs.len()));
        }
        let mut used = vec![false; ours.len()];
        let mut missing = 0;
        for (x, y, text) in &theirs {
            let best = ours
                .iter()
                .enumerate()
                .filter(|(i, _)| !used[*i])
                .map(|(i, (gx, gy))| (i, (gx - x).abs().max((gy - y).abs())))
                .min_by(|a, b| a.1.total_cmp(&b.1));
            match best {
                Some((i, d)) if d <= TOL => {
                    used[i] = true;
                    worst = worst.max(d);
                }
                other => {
                    missing += 1;
                    if missing <= 6 {
                        let near = other.map(|(i, d)| format!(" (nearest ({:.3}, {:.3}) at {d:.3})", ours[i].0, ours[i].1)).unwrap_or_default();
                        failures.push(format!("{name} p{number}: no glyph at {text:?} ({x:.3}, {y:.3}){near}"));
                    }
                }
            }
        }
        if missing > 6 {
            failures.push(format!("{name} p{number}: ... {} more glyphs unmatched", missing - 6));
        }
        let theirs_rules: Vec<[f64; 4]> = ref_rules
            .iter()
            .filter_map(|r| r.as_arr())
            .filter(|r| num(&r[0]) as usize == number)
            .map(|r| [num(&r[1]), num(&r[2]), num(&r[3]), num(&r[4])])
            .collect();
        let mut ours_rules = merge(rules);
        if ours_rules.len() != theirs_rules.len() {
            failures.push(format!("{name} p{number}: {} rules vs reference {}", ours_rules.len(), theirs_rules.len()));
        }
        for r in &theirs_rules {
            let best = ours_rules
                .iter()
                .enumerate()
                .map(|(i, o)| (i, (0..4).map(|k| (o[k] - r[k]).abs()).fold(0.0, f64::max)))
                .min_by(|a, b| a.1.total_cmp(&b.1));
            match best {
                Some((i, d)) if d <= TOL => {
                    worst = worst.max(d);
                    ours_rules.remove(i);
                }
                _ => failures.push(format!("{name} p{number}: no rule within {TOL} bp of {r:?}")),
            }
        }
    }
    (failures, worst)
}

fn fixtures(dir: &str) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|n| n.ends_with(".tex") && n.as_bytes()[0].is_ascii_digit())
        .map(|n| n.trim_end_matches(".tex").to_string())
        .collect();
    names.sort();
    names
}

#[test]
fn verbatim_fixtures_match_pdflatex() {
    if !common::lm_available() {
        eprintln!("SKIP verbatim_oracle: Latin Modern fonts not installed");
        return;
    }
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/verbatim");
    let fonts = FontSet::with_default_dirs(&[]);
    let report = std::env::var("VERBATIM_ORACLE_REPORT").is_ok();
    let names = fixtures(dir);
    assert!(names.len() >= 25, "expected at least 25 verbatim fixtures, found {}", names.len());
    let mut failures = Vec::new();
    let with_compiler = compiler_lexes_lstinline();
    if !with_compiler {
        eprintln!("verbatim_oracle: vendor/compiler does not lex \\lstinline yet; not gating {NEEDS_COMPILER:?}");
    }
    for name in &names {
        let passing = PASSING.contains(&name.as_str()) || (with_compiler && NEEDS_COMPILER.contains(&name.as_str()));
        if !passing && !report {
            continue;
        }
        let (f, worst) = compare(name, dir, &fonts);
        if report {
            eprintln!("{name}: {} ({} mismatches, worst matched delta {worst:.4} bp)", if f.is_empty() { "PASS" } else { "FAIL" }, f.len());
            for line in f.iter().take(12) {
                eprintln!("    {line}");
            }
        }
        if passing {
            failures.extend(f);
        }
    }
    assert!(failures.is_empty(), "{} mismatches:\n{}", failures.len(), failures.join("\n"));
}
