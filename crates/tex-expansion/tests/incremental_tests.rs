//! Incremental re-expansion must be observably identical to expanding
//! the edited source from scratch: same tokens (kind *and* span), same
//! labels, same diagnostics. Exercised with random edits (a fixed-seed
//! xorshift PRNG, no extra dependencies) over every oracle document and
//! the HW1/HW2 real-world fixtures.

use flashtex_tex_expansion::{expand_str, Edit, IncrementalExpander, Limits};
use serde_json::Value;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

const SNIPPETS: &[&str] = &[
    "a", "Z", " ", "\n", "\n\n", "%", "{", "}", "\\relax ", "\\x", "\\def\\x{Q}", "\\newcommand{\\y}[1]{<#1>}",
    "\\y{v}", "\\begingroup", "\\endgroup", "\\iftrue", "\\fi", "\\else", "#", "\\par", "\\stepcounter{section}",
    "\\label{k}", "$x^2$", "\\section{S}",
];

fn floor_char_boundary(s: &str, mut i: usize) -> usize {
    i = i.min(s.len());
    while !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn random_edit(rng: &mut Rng, src: &str) -> Edit {
    let start = floor_char_boundary(src, rng.below(src.len() + 1));
    match rng.below(3) {
        0 => Edit { start, end: start, replacement: SNIPPETS[rng.below(SNIPPETS.len())].to_string() },
        1 => {
            let end = floor_char_boundary(src, start + rng.below(12));
            Edit { start, end, replacement: String::new() }
        }
        _ => {
            let end = floor_char_boundary(src, start + rng.below(4));
            Edit { start, end, replacement: SNIPPETS[rng.below(SNIPPETS.len())].to_string() }
        }
    }
}

/// Applies `edits` random edits and checks equivalence after each one.
/// Returns how many edits converged with the previous run early.
fn check_doc(name: &str, src: &str, edits: usize, interval: usize, seed: u64) -> usize {
    let limits = Limits { max_expansion_steps: 200_000, ..Limits::default() };
    let mut inc = IncrementalExpander::with_options(src, limits, interval);
    let mut rng = Rng(seed | 1);
    let mut converged = 0;
    for i in 0..edits {
        let edit = random_edit(&mut rng, inc.source());
        let stats = inc.edit(&edit);
        if stats.converged_at.is_some() {
            converged += 1;
        }
        let full = {
            let mut e = flashtex_tex_expansion::Engine::with_limits(inc.source(), limits);
            let tokens = e.run();
            (tokens, e.take_diagnostics(), e.take_labels())
        };
        let ctx = || format!("{name}: edit #{i} {edit:?}\nsource now: {:?}", inc.source());
        assert_eq!(inc.tokens(), &full.0[..], "tokens differ -- {}", ctx());
        assert_eq!(inc.labels(), &full.2[..], "labels differ -- {}", ctx());
        let inc_diags: Vec<_> = inc.diagnostics().iter().map(|d| (d.message.clone(), d.span)).collect();
        let full_diags: Vec<_> = full.1.iter().map(|d| (d.message.clone(), d.span)).collect();
        assert_eq!(inc_diags, full_diags, "diagnostics differ -- {}", ctx());
    }
    converged
}

#[test]
fn incremental_matches_full_on_oracle_documents() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/oracle/manifest.json");
    let manifest: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    for (i, (id, case)) in manifest.as_object().unwrap().iter().enumerate() {
        let src = format!("{}{}", case["setup"].as_str().unwrap_or_default(), case["expr"].as_str().unwrap_or_default());
        check_doc(id, &src, 12, 8, 0x9E37_79B9 + i as u64);
    }
}

#[test]
fn incremental_matches_full_on_hw_fixtures() {
    for (name, text) in [("HW1", include_str!("fixtures/HW1.tex")), ("HW2", include_str!("fixtures/HW2.tex"))] {
        // Sanity: the fixture expands at all.
        assert!(!expand_str(text).tokens.is_empty(), "{name} produced no tokens");
        let mut converged = 0;
        for seed in 1..=4u64 {
            converged += check_doc(name, text, 60, 256, seed * 7919);
        }
        assert!(converged > 0, "{name}: no edit ever converged with the previous run");
    }
}

#[test]
fn incremental_matches_full_on_macro_heavy_document() {
    let mut doc = String::from("\\newcounter{c}\\def\\m#1{[#1]}\n");
    for i in 0..200 {
        doc.push_str(&format!("Paragraph {i} \\m{{{i}}} \\stepcounter{{c}}\\arabic{{c}}.\n"));
        if i % 7 == 0 {
            doc.push_str("{\\def\\m#1{(#1)}\\m{g}}\n\n");
        }
        if i % 13 == 0 {
            doc.push_str("\\iffalse skipped \\else taken\\fi\n");
        }
    }
    let converged = check_doc("macro-heavy", &doc, 150, 128, 42);
    assert!(converged > 0);
}
