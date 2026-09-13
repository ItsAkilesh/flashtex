//! Oracle conformance tests: compares this crate's expansion engine
//! against real TeX/e-TeX/pdfLaTeX output, captured ahead of time by
//! `tests/oracle/gen/cases.py` (which shells out to the actual `tex`,
//! `etex`, and `pdflatex` binaries -- oracle-only, per KC-101; this test
//! binary never invokes TeX itself, it only reads the committed
//! `tests/oracle/manifest.json` fixture).
//!
//! To regenerate the fixture after adding cases, run (with MacTeX/TeX Live
//! on PATH):
//!   python3 tests/oracle/gen/cases.py

use flashtex_tex_expansion::{expand_str, tokens_to_display_string};
use serde_json::Value;
use std::fs;

fn manifest() -> Value {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/oracle/manifest.json");
    let text = fs::read_to_string(path).expect("tests/oracle/manifest.json missing -- run tests/oracle/gen/cases.py");
    serde_json::from_str(&text).expect("manifest.json is not valid JSON")
}

#[test]
fn oracle_corpus_matches_real_tex() {
    let manifest = manifest();
    let cases = manifest.as_object().expect("manifest.json root must be an object");
    assert!(cases.len() >= 60, "expected a substantial oracle corpus, found {}", cases.len());

    let mut failures = Vec::new();
    for (id, case) in cases {
        let setup = case["setup"].as_str().unwrap_or_default();
        let expr = case["expr"].as_str().unwrap_or_default();
        let expected = case["expected"].as_str().unwrap_or_default();
        let source = format!("{setup}{expr}");
        let result = expand_str(&source);
        let actual = tokens_to_display_string(&result.tokens);
        if actual != expected {
            failures.push(format!(
                "{id}: source={source:?}\n    expected: {expected:?}\n    actual:   {actual:?}\n    diagnostics: {:?}",
                result.diagnostics
            ));
        }
    }
    if !failures.is_empty() {
        panic!("{} / {} oracle cases mismatched:\n{}", failures.len(), cases.len(), failures.join("\n"));
    }
}

/// Prints a one-line pass/fail count without panicking on individual
/// mismatches, useful while iterating on the engine (`cargo test
/// oracle_summary -- --nocapture`).
#[test]
fn oracle_summary() {
    let manifest = manifest();
    let cases = manifest.as_object().unwrap();
    let mut pass = 0;
    let mut fail = 0;
    for (_, case) in cases {
        let setup = case["setup"].as_str().unwrap_or_default();
        let expr = case["expr"].as_str().unwrap_or_default();
        let expected = case["expected"].as_str().unwrap_or_default();
        let source = format!("{setup}{expr}");
        let result = expand_str(&source);
        let actual = tokens_to_display_string(&result.tokens);
        if actual == expected {
            pass += 1;
        } else {
            fail += 1;
        }
    }
    println!("oracle: {pass} passed, {fail} failed, {} total", cases.len());
}
