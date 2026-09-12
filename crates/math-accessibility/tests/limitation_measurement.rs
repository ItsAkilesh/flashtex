//! FT-038 rev4, item 2: exact measured limitations, not adjectives.
//!
//! Two corpora are measured here, for two different reasons:
//!
//! 1. `flashtex_math_layout::fixtures::all()` — the real producer identified
//!    in `real_producer_corpus.rs`. It is small (23 expressions) and was
//!    written to exercise math-layout's box-layout engine (fraction
//!    stacking, extensible radicals/braces, style overrides), not to stress
//!    accessibility symbol coverage, so it is measured here for completeness
//!    but is not where this crate's real gaps show up.
//! 2. `symbol_coverage_stress_corpus()` below — a corpus this agent built by
//!    hand from common real-world LaTeX math notation (blackboard-bold
//!    number sets, quantifiers, turnstiles, primes, ellipses, degree signs,
//!    the ring accent, ...) specifically because the search for a usable
//!    corpus under `tests/` and `protocol/fixtures` came up empty for this
//!    crate's input type:
//!      - `tests/tex-corpus/cases/math-inline-display/main.tex` has real
//!        LaTeX math source (`$x_1^2 + y$`, `\frac{a+b}{c}=d`), but turning
//!        LaTeX source into a `flashtex_math_layout::MathList` requires a
//!        parser, and the only one in the repo is `crates/compiler`'s
//!        (targeting its own, unrelated `MathList` type — see
//!        `real_producer_corpus.rs`); this agent may not depend on that
//!        crate.
//!      - `protocol/fixtures/*.json` and `crates/rendering-core/tests/fixtures/
//!        {math,display-math}-reference/*` are compile-request/PDF-pixel
//!        comparison fixtures for a different crate's rendering pipeline,
//!        not `MathList` data.
//!
//! So corpus 2 is **explicitly a stand-in**, not a real producer: every
//! symbol in it is a real character used in ordinary mathematical writing,
//! but the `MathList` values are hand-assembled here with the public
//! `Atom`/`MathList` constructors, the same way math-layout's own fixtures
//! are, specifically to give the unsupported-symbol tables something
//! realistic to fail on.
//!
//! Every count below was produced by this file's own test run against
//! `flashtex-math-layout` at the commit recorded in
//! `coordination/agents/daniel-math-access.json`; if either corpus changes
//! shape, the length assertions below fail loudly rather than silently
//! reporting a stale table.

use flashtex_math_accessibility::{MathAccessibility, UnsupportedReason};
use flashtex_math_layout::{Atom, MathList, fixtures};
use std::collections::BTreeMap;

/// Hand-built (see module docs): common real-world math notation that the
/// symbol-name tables in `src/lib.rs` (`named_symbol`, `accent_name`) were
/// not written against. Grouped by the LaTeX macro each expression models,
/// purely for readability of the corpus; the describer only ever sees the
/// resulting `MathList`.
fn symbol_coverage_stress_corpus() -> Vec<(&'static str, MathList)> {
    fn sym(s: &str) -> MathList {
        MathList::symbols(s)
    }

    vec![
        // Blackboard-bold number sets: extremely common, zero coverage.
        ("\\mathbb{R}", MathList::from(Atom::ord('\u{211D}'))),
        ("\\mathbb{N}", MathList::from(Atom::ord('\u{2115}'))),
        ("\\mathbb{Z}", MathList::from(Atom::ord('\u{2124}'))),
        ("\\mathbb{Q}", MathList::from(Atom::ord('\u{211A}'))),
        ("\\mathbb{C}", MathList::from(Atom::ord('\u{2102}'))),
        // Quantifiers, negation, empty set.
        (
            "\\forall x\\, \\exists y\\, (x \\in \\mathbb{R})",
            MathList::new(vec![
                Atom::symbol('\u{2200}'),
                Atom::ord('x'),
                Atom::symbol('\u{2203}'),
                Atom::ord('y'),
                Atom::open('('),
                Atom::ord('x'),
                Atom::rel('\u{2208}'),
                Atom::ord('\u{211D}'),
                Atom::close(')'),
            ]),
        ),
        (
            "\\neg P \\vdash Q",
            MathList::new(vec![
                Atom::symbol('\u{00AC}'),
                Atom::ord('P'),
                Atom::rel('\u{22A2}'),
                Atom::ord('Q'),
            ]),
        ),
        (
            "A \\subseteq \\mathbb{N},\\ A \\neq \\varnothing",
            MathList::new(vec![
                Atom::ord('A'),
                Atom::rel('\u{2286}'),
                Atom::ord('\u{2115}'),
                Atom::punct(','),
                Atom::ord('A'),
                Atom::rel('\u{2260}'),
                Atom::symbol('\u{2205}'),
            ]),
        ),
        // Prime, script ell, reduced Planck constant.
        (
            "f'(x) = \\ell",
            MathList::new(vec![
                Atom::ord('f'),
                Atom::symbol('\u{2032}'),
                Atom::open('('),
                Atom::ord('x'),
                Atom::close(')'),
                Atom::rel('='),
                Atom::ord('\u{2113}'),
            ]),
        ),
        (
            "\\hbar \\ll M \\gg 0",
            MathList::new(vec![
                Atom::ord('\u{210F}'),
                Atom::rel('\u{226A}'),
                Atom::ord('M'),
                Atom::rel('\u{226B}'),
                Atom::symbol('0'),
            ]),
        ),
        // maps-to, circled-dot, congruent.
        (
            "x \\mapsto x^2",
            MathList::new(vec![
                Atom::ord('x'),
                Atom::rel('\u{21A6}'),
                Atom::symbol('x').with_sup(sym("2")),
            ]),
        ),
        (
            "a \\odot b \\cong c",
            MathList::new(vec![
                Atom::ord('a'),
                Atom::bin('\u{2299}'),
                Atom::ord('b'),
                Atom::rel('\u{2245}'),
                Atom::ord('c'),
            ]),
        ),
        // Ellipses (horizontal and vertical) and the degree sign.
        (
            "1, 2, \\dots, n",
            MathList::new(vec![
                Atom::symbol('1'),
                Atom::punct(','),
                Atom::symbol('2'),
                Atom::punct(','),
                Atom::symbol('\u{2026}'),
                Atom::punct(','),
                Atom::ord('n'),
            ]),
        ),
        (
            "a_1 \\vdots a_n",
            MathList::new(vec![
                Atom::symbol('a').with_sub(sym("1")),
                Atom::symbol('\u{22EE}'),
                Atom::symbol('a').with_sub(sym("n")),
            ]),
        ),
        (
            "30^\\circ",
            MathList::new(vec![
                Atom::symbol('3'),
                Atom::symbol('0').with_sup(sym("\u{00B0}")),
            ]),
        ),
        // The ring accent (`\mathring{a}`): a real accent this crate has
        // never named.
        (
            "\\mathring{a}",
            MathList::from(Atom::accent('\u{030A}', sym("a"))),
        ),
        // Not-subset / not-in.
        (
            "A \\not\\subset B,\\ x \\notin S",
            MathList::new(vec![
                Atom::ord('A'),
                Atom::rel('\u{2284}'),
                Atom::ord('B'),
                Atom::punct(','),
                Atom::ord('x'),
                Atom::rel('\u{2209}'),
                Atom::ord('S'),
            ]),
        ),
    ]
}

#[derive(Default)]
struct Tally {
    total_nodes: usize,
    total_unsupported: usize,
    fully_supported_expressions: usize,
    offenders: BTreeMap<String, usize>,
}

fn offender_key(reason: &UnsupportedReason) -> String {
    match reason {
        UnsupportedReason::UnknownSymbolName(c) => {
            format!("unknown-symbol-name U+{:04X} {:?}", *c as u32, c)
        }
        UnsupportedReason::UnknownAccentName(c) => {
            format!("unknown-accent-name U+{:04X} {:?}", *c as u32, c)
        }
        UnsupportedReason::ControlCharacter(c) => {
            format!("control-character U+{:04X}", *c as u32)
        }
    }
}

fn measure(corpus: &[(&'static str, MathList)]) -> Tally {
    let describer = MathAccessibility::new();
    let mut tally = Tally::default();
    for (name, list) in corpus {
        let desc = describer
            .describe(list)
            .unwrap_or_else(|e| panic!("{name:?} unexpectedly exceeded a bound: {e}"));
        tally.total_nodes += desc.nodes.len();
        tally.total_unsupported += desc.unsupported.len();
        if desc.is_fully_supported() {
            tally.fully_supported_expressions += 1;
        }
        for u in &desc.unsupported {
            *tally.offenders.entry(offender_key(&u.reason)).or_insert(0) += 1;
        }
    }
    tally
}

/// The real producer (see `real_producer_corpus.rs`), measured: every symbol
/// math-layout's 23 fixtures use already has a spoken name, so this corpus
/// is fully supported end to end. That is itself the measurement — it shows
/// the gap is not in node-kind coverage (all 11 `Nucleus` variants are
/// handled unconditionally in `render_nucleus`; there is no "unsupported
/// node kind" category at all) but purely in the *symbol name* tables, which
/// this narrow, layout-focused corpus does not exercise.
#[test]
fn real_producer_corpus_is_fully_supported() {
    let corpus = fixtures::all();
    assert_eq!(corpus.len(), 23);
    let tally = measure(&corpus);
    assert_eq!(tally.fully_supported_expressions, 23);
    assert_eq!(tally.total_unsupported, 0);
    assert_eq!(tally.total_nodes, 113);
}

/// The stand-in stress corpus, measured exactly.
#[test]
fn symbol_coverage_stress_corpus_measured_limitations() {
    let corpus = symbol_coverage_stress_corpus();
    assert_eq!(corpus.len(), 17);
    let tally = measure(&corpus);

    // Exact baseline measured against flashtex-math-layout at the commit
    // recorded in coordination/agents/daniel-math-access.json. A change here
    // is a real signal (the tables improved, or the corpus changed) and
    // should be re-measured, not silently widened.
    assert_eq!(tally.total_nodes, 74, "total described nodes changed");
    assert_eq!(
        tally.total_unsupported, 26,
        "unsupported-node count changed: {:#?}",
        tally.offenders
    );
    assert_eq!(
        tally.fully_supported_expressions, 0,
        "every expression in this stress corpus was deliberately built with \
         at least one symbol/accent this crate cannot yet name"
    );

    // 24 distinct offenders account for the 26 hits: `\mathbb{N}` (U+2115)
    // and `\mathbb{R}` (U+211D) each occur twice (once written standalone,
    // once inside a quantifier/subset expression) and are the joint top
    // offenders by frequency; every other gap is a distinct symbol hit once.
    assert_eq!(tally.offenders.len(), 24, "{:#?}", tally.offenders);
    let unknown_symbol = tally
        .offenders
        .keys()
        .filter(|k| k.starts_with("unknown-symbol-name"))
        .count();
    let unknown_accent = tally
        .offenders
        .keys()
        .filter(|k| k.starts_with("unknown-accent-name"))
        .count();
    assert_eq!(
        unknown_symbol, 23,
        "unknown-symbol-name category count changed: {:#?}",
        tally.offenders
    );
    assert_eq!(
        unknown_accent, 1,
        "unknown-accent-name category count changed: {:#?}",
        tally.offenders
    );

    // Top offenders by frequency: blackboard-bold N and R, tied at 2 hits
    // each (every other unsupported symbol in this corpus occurs once).
    for ch in ['\u{2115}', '\u{211D}'] {
        let key = offender_key(&UnsupportedReason::UnknownSymbolName(ch));
        assert_eq!(
            tally.offenders.get(&key).copied(),
            Some(2),
            "blackboard-bold letter {ch:?} should be the joint top offender at 2 hits"
        );
    }
    for ch in ['\u{2124}', '\u{211A}', '\u{2102}'] {
        let key = offender_key(&UnsupportedReason::UnknownSymbolName(ch));
        assert_eq!(
            tally.offenders.get(&key).copied(),
            Some(1),
            "blackboard-bold letter {ch:?} should be exactly one unsupported hit"
        );
    }
}
