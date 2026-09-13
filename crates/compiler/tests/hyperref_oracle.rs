//! hyperref against pdfLaTeX: every `tests/hyperref_oracle/cases/*.tex` has an
//! `expected/*.json` extracted from MacTeX pdflatex (pdfTeX 1.40.29, hyperref
//! 7.01p) by `scripts/hyperref_oracle.py`. No TeX runs here.
//!
//! Compared exactly, per case:
//! - destination names (`Doc-Start`/`page.<n>` excluded: every hyperref PDF
//!   has them and they are per page, not per construct);
//! - the link sequence: each link's target (named destination or URI), its
//!   `/Border` and `/C`, and (single-line links) its visible text;
//! - the outline: titles, tree depth, destinations, `/Count`;
//! - Info strings, `/PageMode`, `/OpenAction`;
//! - with `colorlinks`, the distinct fill operators of the link text.
//!
//! Not asserted here: word positions and link rectangles. The compiler's own
//! layout (`layout.rs`, Core 14 metrics) is not the pdfTeX-metric pipeline;
//! positions belong to `crates/render-pipeline` once it adopts these records
//! (see `docs/proposals/display-list-v2-links.md`). The expected files keep
//! them for that comparison.

use flashtex_compiler::hyperref::{LinkTarget, Options};
use flashtex_compiler::incremental::{compile_full, LayoutConstraints};
use flashtex_compiler::json::{self, Value};
use flashtex_compiler::parser::{self, Block, Inline};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

fn oracle_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/hyperref_oracle")
}

fn obj(v: &Value) -> &BTreeMap<String, Value> {
    match v {
        Value::Obj(map) => map,
        other => panic!("expected object, got {other:?}"),
    }
}

fn arr(v: &Value) -> &[Value] {
    v.as_arr().map(Vec::as_slice).unwrap_or(&[])
}

fn strings(v: Option<&Value>) -> Vec<String> {
    v.map(arr)
        .unwrap_or(&[])
        .iter()
        .map(|s| s.as_str().unwrap_or_default().to_string())
        .collect()
}

fn num(v: &Value) -> f64 {
    match v {
        Value::Str(s) => s.parse().expect("number string"),
        Value::Num(n) => *n,
        other => panic!("expected number, got {other:?}"),
    }
}

struct ExpectedLink {
    target: String,
    border: String,
    color: String,
    text: String,
    lines: usize,
}

/// Annotations in `/Annots` order; a link broken across lines is one
/// annotation per line (pdfTeX), merged back here when the next rectangle
/// with the same target starts a lower line.
fn expected_links(expected: &Value) -> Vec<ExpectedLink> {
    let mut out: Vec<ExpectedLink> = Vec::new();
    let mut previous_rect: Option<[f64; 4]> = None;
    for page in arr(expected.get("pages").unwrap()) {
        previous_rect = previous_rect.filter(|_| false);
        for annotation in arr(page.get("annotations").unwrap()) {
            let target = annotation
                .get("destination")
                .or_else(|| annotation.get("uri"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let rect: Vec<f64> = arr(annotation.get("rect").unwrap())
                .iter()
                .map(num)
                .collect();
            let rect = [rect[0], rect[1], rect[2], rect[3]];
            let text = annotation
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let continues = match (out.last(), previous_rect) {
                // A lower line that starts at or left of the previous piece.
                (Some(last), Some(prev)) => {
                    last.target == target && rect[3] < prev[1] && rect[0] <= prev[0] + 0.01
                }
                _ => false,
            };
            if continues {
                let last = out.last_mut().unwrap();
                last.lines += 1;
                last.text.push(' ');
                last.text.push_str(&text);
            } else {
                out.push(ExpectedLink {
                    target,
                    border: strings(annotation.get("border")).join(" "),
                    color: strings(annotation.get("color")).join(" "),
                    text,
                    lines: 1,
                });
            }
            previous_rect = Some(rect);
        }
    }
    out
}

fn collect_labels(inlines: &[Inline], out: &mut BTreeMap<String, String>) {
    for inline in inlines {
        match inline {
            Inline::Label { key, anchor, .. } => {
                out.insert(key.clone(), anchor.clone());
            }
            Inline::Footnote {
                text: Some(text), ..
            } => collect_labels(text, out),
            _ => {}
        }
    }
}

fn label_anchors(blocks: &[Block]) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for block in blocks {
        match block {
            Block::Paragraph(content)
            | Block::Heading { content, .. }
            | Block::FigureCaption { content }
            | Block::Styled { content, .. }
            | Block::ListItem { content, .. } => collect_labels(content, &mut out),
            _ => {}
        }
    }
    out
}

fn squash(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Outline depth (1 = top) from hyperref levels: a bookmark's parent is the
/// nearest earlier bookmark with a smaller level.
fn depths(levels: &[i32]) -> Vec<usize> {
    let mut stack: Vec<i32> = Vec::new();
    levels
        .iter()
        .map(|&level| {
            while stack.last().is_some_and(|&top| top >= level) {
                stack.pop();
            }
            stack.push(level);
            stack.len()
        })
        .collect()
}

/// `/Count`: open items count every visible descendant, closed items
/// minus their direct children (pdfTeX with hyperref's `bookmarksopen`).
fn counts(depths: &[usize], open: bool) -> Vec<Option<i64>> {
    (0..depths.len())
        .map(|i| {
            let descendants: Vec<usize> = depths[i + 1..]
                .iter()
                .copied()
                .take_while(|&d| d > depths[i])
                .collect();
            if descendants.is_empty() {
                return None;
            }
            if open {
                Some(descendants.len() as i64)
            } else {
                Some(-(descendants.iter().filter(|&&d| d == depths[i] + 1).count() as i64))
            }
        })
        .collect()
}

fn check_case(name: &str, source: &str, expected: &Value) -> Vec<String> {
    let mut failures = Vec::new();
    let mut fail = |message: String| failures.push(format!("{name}: {message}"));
    let parsed = parser::parse(source);
    let hyperref = &parsed.hyperref;
    let options: &Options = &hyperref.options;
    if !hyperref.loaded {
        fail("hyperref not recorded as loaded".into());
    }
    for diagnostic in &parsed.diagnostics {
        let message = &diagnostic.message;
        if message.contains("hyperref")
            || message.contains("not supported")
            || message.contains("environment 'table'")
            || message.contains("environment 'NoHyper'")
        {
            fail(format!("diagnostic: {message}"));
        }
    }

    // Destinations.
    let expected_names: BTreeSet<String> = obj(expected.get("destinations").unwrap())
        .keys()
        .filter(|name| *name != "Doc-Start" && !name.starts_with("page."))
        .cloned()
        .collect();
    let names: BTreeSet<String> = hyperref.anchors.iter().map(|a| a.name.clone()).collect();
    if names != expected_names {
        fail(format!(
            "destinations {names:?} != pdflatex {expected_names:?}"
        ));
    }

    // Links.
    let labels = label_anchors(&parsed.blocks);
    let output = compile_full(source, LayoutConstraints::default());
    let items: Vec<_> = output.pages.iter().flat_map(|page| &page.items).collect();
    let links: Vec<_> = hyperref
        .links
        .iter()
        .filter_map(|link| {
            let target = match &link.target {
                LinkTarget::Destination(name) => name.clone(),
                LinkTarget::Uri(uri) => uri.clone(),
                LinkTarget::Label(key) => labels.get(key)?.clone(),
            };
            let text: String = items
                .iter()
                .filter(|item| item.span.start >= link.span.start && item.span.end <= link.span.end)
                .map(|item| item.text.as_str())
                .collect();
            Some((link, target, text))
        })
        .collect();
    let expected = expected_links(expected);
    let got: Vec<&str> = links.iter().map(|(_, target, _)| target.as_str()).collect();
    let want: Vec<&str> = expected.iter().map(|link| link.target.as_str()).collect();
    if got != want {
        fail(format!("link targets {got:?} != pdflatex {want:?}"));
    } else {
        for ((link, _, text), want) in links.iter().zip(&expected) {
            if options.border() != want.border {
                fail(format!(
                    "{}: /Border [{}] != [{}]",
                    want.target,
                    options.border(),
                    want.border
                ));
            }
            if options.border_color(link.kind) != want.color {
                fail(format!(
                    "{}: /C [{}] != [{}]",
                    want.target,
                    options.border_color(link.kind),
                    want.color
                ));
            }
            let want_text = squash(want.text.trim_end_matches(['.', ',', ';']));
            if want.lines == 1 && squash(text) != want_text {
                fail(format!(
                    "{}: link text {text:?} != pdflatex {:?}",
                    want.target, want.text
                ));
            }
        }
    }

    // colorlinks text colour.
    let mut want_fills: Vec<String> = Vec::new();
    for page in arr(expected_value(expected_path(name)).get("pages").unwrap()) {
        for fill in strings(page.get("fill_colors")) {
            if !want_fills.contains(&fill) {
                want_fills.push(fill);
            }
        }
    }
    let mut fills: Vec<String> = Vec::new();
    for (link, _, _) in &links {
        if let Some(color) = options.text_color(link.kind) {
            let op = color.fill_operator();
            if !fills.contains(&op) {
                fills.push(op);
            }
        }
    }
    if fills != want_fills {
        fail(format!(
            "link fill colours {fills:?} != pdflatex {want_fills:?}"
        ));
    }

    // Outline.
    let whole = expected_value(expected_path(name));
    let outlines = arr(whole.get("outlines").unwrap());
    let levels: Vec<i32> = hyperref.bookmarks.iter().map(|b| b.level).collect();
    let depth = depths(&levels);
    let count = counts(&depth, options.bookmarks_open);
    let got: Vec<(String, usize, String, Option<i64>)> = hyperref
        .bookmarks
        .iter()
        .zip(depth.iter().zip(&count))
        .map(|(b, (d, c))| (b.title.clone(), *d, b.destination.clone(), *c))
        .collect();
    let want: Vec<(String, usize, String, Option<i64>)> = outlines
        .iter()
        .map(|o| {
            (
                o.get("title")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                o.get("level").and_then(Value::as_i64).unwrap_or_default() as usize,
                o.get("destination")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                o.get("count").and_then(Value::as_i64),
            )
        })
        .collect();
    if got != want {
        fail(format!("outline {got:?} != pdflatex {want:?}"));
    }

    // Info and catalog.
    let info = obj(whole.get("info").unwrap());
    let field = |key: &str| {
        info.get(key)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    };
    let ours = [
        ("Title", options.pdftitle.clone().unwrap_or_default()),
        ("Author", options.pdfauthor.clone().unwrap_or_default()),
        ("Subject", options.pdfsubject.clone().unwrap_or_default()),
        ("Keywords", options.pdfkeywords.clone().unwrap_or_default()),
        (
            "Creator",
            options
                .pdfcreator
                .clone()
                .unwrap_or_else(|| "LaTeX with hyperref".to_string()),
        ),
    ];
    for (key, value) in ours {
        if field(key) != value {
            fail(format!("/{key} ({value}) != pdflatex ({})", field(key)));
        }
    }
    let catalog = whole.get("catalog").unwrap();
    let page_mode = if options.bookmarks {
        "UseOutlines"
    } else {
        "UseNone"
    };
    if catalog.get("page_mode").and_then(Value::as_str) != Some(page_mode) {
        fail(format!(
            "/PageMode {page_mode} != pdflatex {:?}",
            catalog.get("page_mode")
        ));
    }
    let open_action: Vec<String> = arr(catalog.get("open_action").unwrap())
        .iter()
        .map(|v| match v {
            Value::Num(n) => format!("{n}"),
            other => other.as_str().unwrap_or_default().to_string(),
        })
        .collect();
    if open_action != ["1", "Fit"] {
        fail(format!("/OpenAction {open_action:?} != [1 Fit]"));
    }
    failures
}

fn expected_path(name: &str) -> PathBuf {
    oracle_dir().join("expected").join(format!("{name}.json"))
}

fn expected_value(path: PathBuf) -> Value {
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    json::parse(&text).unwrap_or_else(|e| panic!("{}: {e:?}", path.display()))
}

#[test]
fn hyperref_matches_pdflatex_on_every_oracle_case() {
    let mut cases: Vec<PathBuf> = std::fs::read_dir(oracle_dir().join("cases"))
        .expect("cases directory")
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "tex"))
        .collect();
    cases.sort();
    assert!(cases.len() >= 25, "only {} oracle cases", cases.len());
    let mut failures = Vec::new();
    for case in &cases {
        let name = case.file_stem().unwrap().to_str().unwrap().to_string();
        let source = std::fs::read_to_string(case).unwrap();
        let expected = expected_value(expected_path(&name));
        assert_eq!(
            expected.get("case").and_then(Value::as_str),
            Some(format!("{name}.tex").as_str()),
            "stale expected file for {name}; run scripts/hyperref_oracle.py"
        );
        failures.extend(check_case(&name, &source, &expected));
    }
    assert!(
        failures.is_empty(),
        "{} mismatches:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn outline_depths_and_counts_follow_pdftex() {
    // 22-bookmarks-tree: One, One A, One A i, One B, Two, Two A.
    let depth = depths(&[1, 2, 3, 2, 1, 2]);
    assert_eq!(depth, vec![1, 2, 3, 2, 1, 2]);
    assert_eq!(
        counts(&depth, false),
        vec![Some(-2), Some(-1), None, None, Some(-1), None]
    );
    // pdfbookmark level 0 after a level-2 entry returns to the top.
    assert_eq!(depths(&[1, 1, 2, 0]), vec![1, 1, 2, 1]);
    assert_eq!(counts(&[1, 2, 1], true), vec![Some(1), None, None]);
}
