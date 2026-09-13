//! List-layout gate: word origins against the pdfLaTeX (MacTeX 2026,
//! pdfTeX 1.40.29) references pinned in `tests/list_corpus/refs` (made by
//! that corpus's `oracle.py refs`, oracle tooling only; cargo never runs
//! TeX).
//!
//! The corpus covers `itemize`/`enumerate`/`description` at 10/11/12pt,
//! one and two columns, nesting to depth 4, consecutive lists, lists after
//! paragraphs, headings and displays, multi-paragraph items, `quote`/
//! `quotation`/`verse`/`center` next to lists and enumitem keys.
//!
//! For every fixture in `PASSING`: the page count matches and every
//! reference word (labels included) has a candidate glyph origin within
//! 0.5 bp in x and y. `oracle.py check` measures all fixtures, including
//! the ones listed in `PENDING` with the reason they do not match yet.

mod common;

use flashtex_compiler::json::{self, Value};
use flashtex_render_pipeline::display::Item;

const WORD_TOL: f64 = 0.5;

const PASSING: &[&str] = &[
    "01-itemize-10pt",
    "01-itemize-11pt",
    "01-itemize-12pt",
    "02-enumerate-10pt",
    "02-enumerate-11pt",
    "02-enumerate-12pt",
    "04-nested-enumerate-11pt",
    "04-nested-enumerate-12pt",
    "04-nested-itemize-10pt",
    "04-nested-mixed-10pt",
    "05-twocolumn-itemize-10pt",
    "05-twocolumn-nested-11pt",
    "06-consecutive-10pt",
    "06-consecutive-blank-12pt",
    "07-after-display-11pt",
    "07-after-heading-10pt",
    "08-multipar-10pt",
    "08-multipar-12pt",
    "10-center-after-list-10pt",
    "10-quotation-10pt",
    "10-quote-11pt",
    "11-enumitem-label-10pt",
    "11-enumitem-labelsep-12pt",
    "11-enumitem-leftmargin-star-11pt",
    "11-enumitem-nosep-10pt",
    "11-enumitem-seps-12pt",
    "11-enumitem-setlist-10pt",
    "12-long-items-11pt",
];

/// Fixtures `oracle.py check` still reports as failing, and why.
#[allow(dead_code)]
const PENDING: &[(&str, &str)] = &[
    ("03-description-10pt", "vendored compiler: no `description` environment or `\\item[<label>]`"),
    ("03-description-11pt", "vendored compiler: no `description`"),
    ("03-description-12pt", "vendored compiler: no `description`"),
    ("05-twocolumn-description-12pt", "vendored compiler: no `description`"),
    ("09-description-in-itemize-10pt", "vendored compiler: no `description`"),
    ("09-description-long-11pt", "vendored compiler: no `description`"),
    ("12-item-optional-10pt", "vendored compiler: `\\item[<label>]` is typeset as text"),
    ("11-enumitem-noitemsep-11pt", "vendored compiler: `[noitemsep]` is read as a shortlabels template (label text)"),
    ("11-enumitem-start-resume-10pt", "vendored compiler: `start=`/`resume` ignored (numbering and label text)"),
    ("10-verse-12pt", "vendored compiler: `verse` is not a paragraph environment; pipeline has no verse geometry"),
    ("10-quote-in-list-11pt", "pipeline: `quote` inside a list does not take the list's margin plus `\\leftmarginii`, nor level-2 `\\topsep`"),
];

fn num(v: &Value) -> f64 {
    match v {
        Value::Num(n) => *n,
        _ => f64::NAN,
    }
}

#[test]
fn list_fixtures_match_pdflatex() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/list_corpus");
    if !common::lm_available() {
        eprintln!("SKIP list_oracle: Latin Modern fonts not installed");
        return;
    }
    let mut failures = Vec::new();
    for name in PASSING {
        let tex = std::fs::read_to_string(format!("{dir}/fixtures/{name}.tex")).unwrap();
        let reference = json::parse(&std::fs::read_to_string(format!("{dir}/refs/{name}.json")).unwrap()).unwrap();
        let ref_pages = reference.get("pages").and_then(|p| p.as_arr()).unwrap();
        let rendered = common::render_one(&tex);
        let pages = &rendered.v2.pages;
        if pages.len() != ref_pages.len() {
            failures.push(format!("{name}: {} pages vs reference {}", pages.len(), ref_pages.len()));
            continue;
        }
        for (page, words) in pages.iter().zip(ref_pages) {
            let glyphs: Vec<(f64, f64)> = page
                .items
                .iter()
                .filter_map(|item| match item {
                    Item::GlyphRun(run) => Some(run.glyphs.iter().map(|g| (g.origin_x.to_bp(), g.baseline_y.to_bp()))),
                    _ => None,
                })
                .flatten()
                .collect();
            for w in words.as_arr().unwrap() {
                let (x, y) = (num(w.get("x").unwrap()), num(w.get("y_top").unwrap()));
                if !glyphs.iter().any(|(gx, gy)| (gx - x).abs() <= WORD_TOL && (gy - y).abs() <= WORD_TOL) {
                    failures.push(format!("{name}: page {}: no glyph at word {:?} ({x:.3}, {y:.3})", page.number, w.get("text").and_then(|t| t.as_str()).unwrap_or("")));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{} mismatches:\n{}", failures.len(), failures.join("\n"));
}
