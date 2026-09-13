//! Inline `\includegraphics` and the graphics box transforms against
//! pdfLaTeX (MacTeX 2026) references in `fixtures/graphics/reference/*.json`
//! (made by `fixtures/graphics/oracle.py` from the PDF content streams;
//! test-only, cargo never runs TeX).
//!
//! Both sides are reduced to the same shape: glyphs in paint order with
//! their page-space origin and advance vector (big points, y down), grouped
//! into words wherever the next origin is not where the previous advance
//! ends (1 bp). Checked per fixture:
//! * every reference word is found (same page, same letters) within 0.5 bp;
//! * the line starts (first word of every horizontal line) are the same;
//! * every image's unit-square transform agrees within 0.01 bp, and its
//!   clip quadrilateral (graphicx `clip`) when there is one;
//! * every filled rule (the `draft` frame) within 0.05 bp.

mod common;

use flashtex_compiler::json::{self, Value};
use flashtex_compiler::parser::SourceDocument;
use flashtex_render_pipeline::display::{DisplayList, Item};
use flashtex_render_pipeline::{render, FontSet, RenderOptions};

const WORD_TOL: f64 = 0.5;
const CTM_TOL: f64 = 0.01;
const RULE_TOL: f64 = 0.05;

fn num(v: &Value, k: &str) -> f64 {
    match v.get(k) {
        Some(Value::Num(n)) => *n,
        _ => f64::NAN,
    }
}

fn arr<'a>(v: &'a Value, k: &str) -> &'a [Value] {
    v.get(k).and_then(|a| a.as_arr()).map_or(&[], |a| a.as_slice())
}

#[derive(Debug, Clone)]
struct Glyph {
    page: u32,
    text: String,
    x: f64,
    y: f64,
    dx: f64,
    dy: f64,
}

#[derive(Debug, Clone)]
struct Word {
    page: u32,
    text: String,
    x: f64,
    y: f64,
    dx: f64,
    dy: f64,
}

/// Letters and digits only, math alphanumerics folded to ASCII.
fn norm(s: &str) -> String {
    s.chars()
        .filter_map(|c| {
            let u = c as u32;
            let c = match u {
                0x1D434..=0x1D44D => char::from_u32('A' as u32 + (u - 0x1D434))?,
                0x1D44E..=0x1D467 => char::from_u32('a' as u32 + (u - 0x1D44E))?,
                0x210E => 'h',
                _ => c,
            };
            c.is_ascii_alphanumeric().then_some(c)
        })
        .collect()
}

fn words(glyphs: &[Glyph]) -> Vec<Word> {
    let mut out: Vec<Word> = Vec::new();
    let mut prev: Option<&Glyph> = None;
    for g in glyphs {
        let joined = prev.is_some_and(|p| p.page == g.page && (p.x + p.dx - g.x).hypot(p.y + p.dy - g.y) < 1.0);
        match out.last_mut() {
            Some(w) if joined => w.text.push_str(&g.text),
            _ => out.push(Word { page: g.page, text: g.text.clone(), x: g.x, y: g.y, dx: g.dx, dy: g.dy }),
        }
        prev = Some(g);
    }
    out
}

fn reference_glyphs(r: &Value) -> Vec<Glyph> {
    arr(r, "glyphs")
        .iter()
        .map(|g| Glyph {
            page: num(g, "page") as u32,
            text: g.get("text").and_then(|t| t.as_str()).unwrap_or("").to_string(),
            x: num(g, "x"),
            y: num(g, "y"),
            dx: num(g, "dx"),
            dy: num(g, "dy"),
        })
        .collect()
}

fn our_glyphs(v2: &DisplayList) -> Vec<Glyph> {
    let mut out = Vec::new();
    for p in &v2.pages {
        for it in &p.items {
            let Item::GlyphRun(run) = it else { continue };
            let mut seen = std::collections::BTreeSet::new();
            for g in &run.glyphs {
                let text = if seen.insert(g.cluster) {
                    run.clusters.get(g.cluster as usize).and_then(|c| run.text.get(c.text_start_byte..c.text_end_byte)).unwrap_or("").to_string()
                } else {
                    String::new()
                };
                out.push(Glyph { page: p.number, text, x: g.origin_x.to_bp(), y: g.baseline_y.to_bp(), dx: g.advance_x.to_bp(), dy: g.advance_y.to_bp() });
            }
        }
    }
    out
}

/// The first word of every horizontal line, in page and baseline order.
fn line_starts(words: &[Word]) -> Vec<(u32, String)> {
    let mut horizontal: Vec<&Word> = words.iter().filter(|w| w.dy.abs() < 1e-3 && w.dx > 0.0 && !norm(&w.text).is_empty()).collect();
    horizontal.sort_by(|a, b| (a.page, a.y).partial_cmp(&(b.page, b.y)).unwrap());
    let mut lines: Vec<Vec<&Word>> = Vec::new();
    for w in horizontal {
        match lines.last_mut() {
            Some(l) if l[0].page == w.page && (l[0].y - w.y).abs() < WORD_TOL => l.push(w),
            _ => lines.push(vec![w]),
        }
    }
    lines
        .into_iter()
        .map(|mut l| {
            l.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
            (l[0].page, norm(&l[0].text))
        })
        .collect()
}

fn quad(t: &[f64; 6], c: [f64; 4]) -> Vec<(f64, f64)> {
    let p = |u: f64, v: f64| (t[0] * u + t[2] * v + t[4], t[1] * u + t[3] * v + t[5]);
    vec![p(c[0], c[1]), p(c[2], c[1]), p(c[2], c[3]), p(c[0], c[3])]
}

fn quad_diff(a: &[(f64, f64)], b: &[(f64, f64)]) -> f64 {
    let key = |p: &(f64, f64)| ((p.0 * 100.0).round() as i64, (p.1 * 100.0).round() as i64);
    let (mut a, mut b) = (a.to_vec(), b.to_vec());
    a.sort_by_key(key);
    b.sort_by_key(key);
    if a.len() != b.len() {
        return f64::INFINITY;
    }
    a.iter().zip(&b).map(|(p, q)| (p.0 - q.0).abs().max((p.1 - q.1).abs())).fold(0.0, f64::max)
}

struct Outcome {
    name: String,
    failures: Vec<String>,
    row: String,
}

fn check(dir: &str, name: &str, fonts: &FontSet) -> Outcome {
    let tex = std::fs::read_to_string(format!("{dir}/{name}")).unwrap();
    let reference = json::parse(&std::fs::read_to_string(format!("{dir}/reference/{}", name.replace(".tex", ".json"))).unwrap()).unwrap();
    let options = RenderOptions { project_root: Some(dir.into()), ..RenderOptions::default() };
    let r = render(&[SourceDocument { path: "main.tex", text: &tex }], "main.tex", 1, "graphics", fonts, &options);
    let v2 = &r.v2;
    let mut failures = Vec::new();
    let ref_pages = num(&reference, "pages") as usize;
    if v2.pages.len() != ref_pages {
        failures.push(format!("pages {} vs reference {ref_pages}", v2.pages.len()));
    }
    for d in &v2.diagnostics {
        if d.code.starts_with("image") || d.code.starts_with("graphics") {
            failures.push(format!("diagnostic {}: {}", d.code, d.message));
        }
    }
    // Words.
    let theirs = words(&reference_glyphs(&reference));
    let ours = words(&our_glyphs(v2));
    let (mut matched, mut total, mut worst) = (0, 0, 0.0f64);
    for w in &theirs {
        let n = norm(&w.text);
        if n.is_empty() {
            continue;
        }
        total += 1;
        let best = ours.iter().filter(|o| o.page == w.page && norm(&o.text) == n).map(|o| (o.x - w.x).abs().max((o.y - w.y).abs())).fold(f64::INFINITY, f64::min);
        if best <= WORD_TOL {
            matched += 1;
            worst = worst.max(best);
        } else {
            failures.push(format!("word {:?} p{} ({:.3},{:.3}): {}", w.text, w.page, w.x, w.y, if best.is_finite() { format!("nearest ours {best:.3} bp away") } else { "not rendered".into() }));
        }
    }
    let our_words = ours.iter().filter(|w| !norm(&w.text).is_empty()).count();
    if our_words != total {
        failures.push(format!("{our_words} words rendered vs reference {total}"));
    }
    let (ls_ours, ls_theirs) = (line_starts(&ours), line_starts(&theirs));
    if ls_ours != ls_theirs {
        failures.push(format!("line starts ours {ls_ours:?} vs reference {ls_theirs:?}"));
    }
    // Images.
    let mut our_images: Vec<(u32, [f64; 6], Option<[f64; 4]>)> = v2
        .pages
        .iter()
        .flat_map(|p| p.items.iter().filter_map(move |it| if let Item::Image(i) = it { Some((p.number, i.transform, i.clip)) } else { None }))
        .collect();
    let ref_images = arr(&reference, "images");
    let (mut img_ok, mut img_worst) = (0, 0.0f64);
    if our_images.len() != ref_images.len() {
        failures.push(format!("{} images vs reference {}", our_images.len(), ref_images.len()));
    }
    for im in ref_images {
        let page = num(im, "page") as u32;
        let t: Vec<f64> = arr(im, "transform").iter().map(|v| if let Value::Num(n) = v { *n } else { f64::NAN }).collect();
        let Some((at, diff)) = our_images
            .iter()
            .enumerate()
            .filter(|(_, o)| o.0 == page)
            .map(|(i, o)| (i, o.1.iter().zip(&t).map(|(a, b)| (a - b).abs()).fold(0.0, f64::max)))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        else {
            failures.push(format!("image {t:?} not rendered"));
            continue;
        };
        let (_, ours_t, ours_clip) = our_images.remove(at);
        let ref_clips = arr(im, "clips");
        let clip_diff = match (ours_clip, ref_clips.first()) {
            (None, None) => 0.0,
            (Some(c), Some(q)) => {
                let theirs: Vec<(f64, f64)> = q.as_arr().unwrap().iter().map(|p| {
                    let p = p.as_arr().unwrap();
                    (if let Value::Num(n) = p[0] { n } else { f64::NAN }, if let Value::Num(n) = p[1] { n } else { f64::NAN })
                }).collect();
                quad_diff(&quad(&ours_t, c), &theirs)
            }
            _ => f64::INFINITY,
        };
        if diff <= CTM_TOL && clip_diff <= CTM_TOL {
            img_ok += 1;
            img_worst = img_worst.max(diff).max(clip_diff);
        } else {
            failures.push(format!("image ours {ours_t:?} clip {ours_clip:?} vs reference {t:?} clips {} (transform diff {diff:.4}, clip diff {clip_diff:.4})", ref_clips.len()));
        }
    }
    // Rules.
    let mut our_rules: Vec<(u32, f64, f64, f64, f64)> = v2
        .pages
        .iter()
        .flat_map(|p| p.items.iter().filter_map(move |it| if let Item::Rule(r) = it { Some((p.number, r.x.to_bp(), r.top.to_bp(), (r.x.0 + r.width.0) as f64 / 1_048_576.0, (r.top.0 + r.height.0) as f64 / 1_048_576.0)) } else { None }))
        .collect();
    let ref_rules = arr(&reference, "rules");
    let mut rules_ok = 0;
    if our_rules.len() != ref_rules.len() {
        failures.push(format!("{} rules vs reference {}", our_rules.len(), ref_rules.len()));
    }
    for rr in ref_rules {
        let page = num(rr, "page") as u32;
        let pts: Vec<(f64, f64)> = arr(rr, "corners").iter().map(|p| {
            let p = p.as_arr().unwrap();
            (if let Value::Num(n) = p[0] { n } else { f64::NAN }, if let Value::Num(n) = p[1] { n } else { f64::NAN })
        }).collect();
        let (x0, y0) = pts.iter().fold((f64::INFINITY, f64::INFINITY), |a, p| (a.0.min(p.0), a.1.min(p.1)));
        let (x1, y1) = pts.iter().fold((f64::NEG_INFINITY, f64::NEG_INFINITY), |a, p| (a.0.max(p.0), a.1.max(p.1)));
        let best = our_rules
            .iter()
            .enumerate()
            .filter(|(_, o)| o.0 == page)
            .map(|(i, o)| (i, [(o.1 - x0).abs(), (o.2 - y0).abs(), (o.3 - x1).abs(), (o.4 - y1).abs()].into_iter().fold(0.0, f64::max)))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        match best {
            Some((i, d)) if d <= RULE_TOL => {
                our_rules.remove(i);
                rules_ok += 1;
            }
            other => failures.push(format!("rule ({x0:.3},{y0:.3})-({x1:.3},{y1:.3}) p{page}: nearest {:?}", other.map(|o| o.1))),
        }
    }
    let row = format!(
        "| {name} | {}/{ref_pages} | {matched}/{total} | {worst:.3} | {img_ok}/{} | {img_worst:.4} | {rules_ok}/{} | {} |",
        v2.pages.len(),
        ref_images.len(),
        ref_rules.len(),
        if ls_ours == ls_theirs { "same" } else { "differ" }
    );
    Outcome { name: name.to_string(), failures, row }
}

#[test]
fn graphics_fixtures_match_pdflatex() {
    if !common::lm_available() {
        eprintln!("SKIP graphics_oracle: Latin Modern fonts not installed");
        return;
    }
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/graphics");
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|n| n.ends_with(".tex") && n.as_bytes()[0].is_ascii_digit())
        .collect();
    names.sort();
    assert_eq!(names.len(), 24, "expected 24 graphics fixtures");
    let fonts = FontSet::with_default_dirs(&[]);
    let mut report = String::from("| fixture | pages ours/ref | words | max word diff (bp) | images | max transform/clip diff (bp) | rules | line starts |\n|---|---|---|---|---|---|---|---|\n");
    let mut failed = Vec::new();
    for name in &names {
        let o = check(dir, name, &fonts);
        report.push_str(&o.row);
        report.push('\n');
        if !o.failures.is_empty() {
            failed.push(format!("{}:\n  {}", o.name, o.failures.join("\n  ")));
        }
    }
    println!("{report}");
    assert!(failed.is_empty(), "{} of {} fixtures differ:\n{}", failed.len(), names.len(), failed.join("\n"));
}

#[test]
fn transform_fields_are_only_serialised_when_negotiated() {
    if !common::lm_available() {
        return;
    }
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/graphics");
    let fonts = FontSet::with_default_dirs(&[]);
    let options = RenderOptions { project_root: Some(dir.into()), ..RenderOptions::default() };
    for (name, field, feature) in [("09-rotate-quarter.tex", "\"glyph_transform\":", "\"glyph_transform\""), ("16-trim-clip.tex", "\"clip\":", "\"image_clip\"")] {
        let tex = std::fs::read_to_string(format!("{dir}/{name}")).unwrap();
        let r = render(&[SourceDocument { path: "main.tex", text: &tex }], "main.tex", 1, "graphics", &fonts, &options);
        let frozen = r.v2.write_json("x");
        assert!(!frozen.contains(field) && !frozen.contains(feature), "{name}: the frozen line must not carry {field}");
        let images = r.v2.write_json_with("x", true);
        assert!(!images.contains(field) && !images.contains(feature), "{name}: display-list-v2-images alone must not carry {field}");
        let negotiated = r.v2.write_json_caps("x", true, true);
        assert!(negotiated.contains(field) && negotiated.contains(feature), "{name}: {field} missing with display-list-v2-transforms");
        assert_eq!(negotiated, json::write(&r.v2.to_json_caps("x", true, true)), "{name}: direct writer differs from the value tree");
    }
}
