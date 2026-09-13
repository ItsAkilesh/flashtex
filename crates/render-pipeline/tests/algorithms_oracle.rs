//! Pseudocode gate: `algorithm` floats, `algorithmic` and `algpseudocode`
//! against pdfLaTeX (MacTeX 2026) references committed in
//! `fixtures/algorithms/reference/*.json` (made by
//! `fixtures/algorithms/oracle.py`, test-only; cargo never runs TeX).
//!
//! Every pdflatex word (`pdftotext -bbox`: a run of glyphs between spaces,
//! so one word may span a formula's atoms) must begin at one of our glyphs
//! with the same first character, on the same page, within 0.5 bp in x and,
//! when the reference knows it, in baseline; every rule must match within
//! 0.1 bp. Fixtures listed in [`KNOWN`] have documented deviations: the
//! test fails if one of them starts to pass (update the list) or if any
//! other fixture fails.

mod common;

use flashtex_compiler::json::{self, Value};
use flashtex_render_pipeline::display::Item;

const WORD_TOL: f64 = 0.5;
const RULE_TOL: f64 = 0.1;

/// Fixtures that do not match yet, with the reason.
const KNOWN: &[(&str, &str)] = &[
    ("09-algpseudocode-basic", "math: `\\bmod` spacing (`\\mkern5mu` + `\\mathbin` mod) is set narrower (math-layout)"),
    ("10-algpseudocode-loops", "math: `\\dots` between commas (`\\ldots` as `\\mathinner`) is spaced differently (math-layout)"),
    ("12-algpseudocode-procedures", "math: `\\bmod` spacing, as 09"),
    ("17-algorithm-ref", "a line right after a ruled float's `\\hrule` is as tall as its glyph boxes, not their TFM heights (6.30pt for `t`, TeX 6.15pt): the rules below drift up to 0.24bp"),
    ("18-algorithm-top", "the same glyph-box line height after the mid rule (`x←1`: 6.30pt, TeX 6.44pt): the bottom rule is 0.10bp high"),
    ("20-algorithmic-bare", "the text line ended by a bare `\\begin{algorithmic}` in horizontal mode is set with shrunk interword glue by pdflatex; the pipeline (also without pseudocode) sets it at natural width. The list lines match"),
];

fn num(v: &Value, k: &str) -> Option<f64> {
    match v.get(k) {
        Some(Value::Num(n)) => Some(*n),
        _ => None,
    }
}

/// Characters extraction spells differently on the two sides.
fn fold(c: char) -> char {
    match c {
        '’' | '‘' => '\'',
        '“' | '”' => '"',
        other => other,
    }
}

struct Atom {
    page: u32,
    x: f64,
    baseline: f64,
    first: char,
}

/// One atom per glyph cluster: its first glyph's origin and the first
/// character of its text.
fn atoms(r: &flashtex_render_pipeline::Rendered) -> Vec<Atom> {
    let mut out = Vec::new();
    for page in &r.v2.pages {
        for it in &page.items {
            let Item::GlyphRun(run) = it else { continue };
            let mut seen = std::collections::BTreeSet::new();
            for g in &run.glyphs {
                if !seen.insert(g.cluster) {
                    continue;
                }
                let Some(cluster) = run.clusters.get(g.cluster as usize) else { continue };
                let Some(first) = run.text.get(cluster.text_start_byte..cluster.text_end_byte).and_then(|t| t.chars().next()) else { continue };
                out.push(Atom { page: page.number, x: g.origin_x.to_bp(), baseline: g.baseline_y.to_bp(), first: fold(first) });
            }
        }
    }
    out
}

struct Outcome {
    failures: Vec<String>,
    words: usize,
    worst_x: f64,
    worst_y: f64,
    rules: usize,
}

fn check(tex: &str, reference: &Value) -> Outcome {
    let r = common::render_one(tex);
    let mut failures = Vec::new();
    let ref_pages = num(reference, "pages").unwrap_or(0.0) as usize;
    if r.v2.pages.len() != ref_pages {
        failures.push(format!("pages {} vs {ref_pages}", r.v2.pages.len()));
    }
    for d in &r.v2.diagnostics {
        if d.code.starts_with("algorithm") || d.code.starts_with("float") || d.code == "unsupported_block" {
            failures.push(format!("diagnostic {}: {}", d.code, d.message));
        }
    }
    let ours = atoms(&r);
    let mut out = Outcome { failures: Vec::new(), words: 0, worst_x: 0.0, worst_y: 0.0, rules: 0 };
    for w in reference.get("words").and_then(|v| v.as_arr()).into_iter().flatten() {
        let text = w.get("text").and_then(|t| t.as_str()).unwrap_or("");
        let Some(first) = text.chars().next().map(fold) else { continue };
        let page = num(w, "page").unwrap_or(0.0) as u32;
        let x = num(w, "x").unwrap_or(f64::NAN);
        let baseline = num(w, "baseline");
        let best = ours
            .iter()
            .filter(|a| a.page == page && a.first == first)
            .map(|a| {
                let dx = (a.x - x).abs();
                let dy = baseline.map_or(0.0, |b| (a.baseline - b).abs());
                (dx + dy, dx, dy, a)
            })
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        match best {
            Some((_, dx, dy, a)) => {
                out.words += 1;
                out.worst_x = out.worst_x.max(dx);
                out.worst_y = out.worst_y.max(dy);
                if dx > WORD_TOL || dy > WORD_TOL {
                    failures.push(format!(
                        "word {text:?} p{page}: ours x {:.3} baseline {:.3} vs x {x:.3} baseline {}",
                        a.x,
                        a.baseline,
                        baseline.map_or("-".to_string(), |b| format!("{b:.3}"))
                    ));
                }
            }
            // A symbol may be extracted as a different character on each
            // side; a letter or digit may not.
            None if first.is_alphanumeric() => failures.push(format!("word {text:?} p{page} x {x:.3}: no glyph")),
            None => {}
        }
    }
    let mut our_rules: Vec<(u32, f64, f64, f64, f64)> = r
        .v2
        .pages
        .iter()
        .flat_map(|p| {
            p.items.iter().filter_map(move |it| match it {
                Item::Rule(rule) => Some((p.number, rule.x.to_bp(), rule.top.to_bp(), rule.width.to_bp(), rule.height.to_bp())),
                _ => None,
            })
        })
        .collect();
    our_rules.sort_by(|a, b| (a.0, a.2).partial_cmp(&(b.0, b.2)).unwrap());
    let theirs: Vec<(u32, f64, f64, f64, f64)> = reference
        .get("rules")
        .and_then(|v| v.as_arr())
        .into_iter()
        .flatten()
        .map(|v| (num(v, "page").unwrap_or(0.0) as u32, num(v, "x").unwrap_or(0.0), num(v, "top").unwrap_or(0.0), num(v, "width").unwrap_or(0.0), num(v, "height").unwrap_or(0.0)))
        .collect();
    if our_rules.len() != theirs.len() {
        failures.push(format!("{} rules vs {}", our_rules.len(), theirs.len()));
    }
    for (o, t) in our_rules.iter().zip(&theirs) {
        out.rules += 1;
        let d = [(o.1 - t.1).abs(), (o.2 - t.2).abs(), (o.3 - t.3).abs(), (o.4 - t.4).abs()].into_iter().fold(0.0, f64::max);
        if o.0 != t.0 || d > RULE_TOL {
            failures.push(format!("rule ours p{} x{:.3} top{:.3} {:.3}x{:.3} vs p{} x{:.3} top{:.3} {:.3}x{:.3}", o.0, o.1, o.2, o.3, o.4, t.0, t.1, t.2, t.3, t.4));
        }
    }
    out.failures = failures;
    out
}

#[test]
fn pseudocode_fixtures_match_pdflatex() {
    if !common::lm_available() {
        eprintln!("SKIP algorithms_oracle: Latin Modern fonts not installed");
        return;
    }
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/algorithms");
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|n| n.ends_with(".tex") && n.as_bytes()[0].is_ascii_digit())
        .collect();
    names.sort();
    assert!(names.len() >= 20, "expected at least 20 pseudocode fixtures, found {}", names.len());
    let mut report = String::new();
    let mut problems = Vec::new();
    let mut exact = 0;
    for name in &names {
        let tex = std::fs::read_to_string(format!("{dir}/{name}")).unwrap();
        let reference = json::parse(&std::fs::read_to_string(format!("{dir}/reference/{}", name.replace(".tex", ".json"))).unwrap()).unwrap();
        let o = check(&tex, &reference);
        let stem = name.trim_end_matches(".tex");
        let known = KNOWN.iter().find(|(k, _)| *k == stem);
        report.push_str(&format!(
            "{stem}: {} words (worst x {:.3} y {:.3}), {} rules, {}\n",
            o.words,
            o.worst_x,
            o.worst_y,
            o.rules,
            if o.failures.is_empty() { "exact".to_string() } else { format!("{} mismatches", o.failures.len()) }
        ));
        if o.failures.is_empty() {
            exact += 1;
            if let Some((_, why)) = known {
                problems.push(format!("{stem} now matches; remove it from KNOWN ({why})"));
            }
        } else if known.is_none() {
            for f in o.failures.iter().take(12) {
                problems.push(format!("{stem}: {f}"));
            }
        }
    }
    eprintln!("{report}pseudocode oracle: {exact}/{} exact", names.len());
    assert!(problems.is_empty(), "{report}\n{}", problems.join("\n"));
}
