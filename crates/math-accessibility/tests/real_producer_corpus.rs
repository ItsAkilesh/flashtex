//! FT-038 rev4, item 1: the actual existing producer of
//! `flashtex_math_layout::MathList` values in this repository, run through
//! this crate's describer.
//!
//! ## What was checked before writing this file
//!
//! `crates/math-accessibility` depends on exactly one crate,
//! `flashtex-math-layout` (see `Cargo.toml`; the boundary for this agent
//! forbids depending on any other crate). So "the actual existing producer"
//! has to mean: what in this repository, reachable from that one dependency,
//! actually builds a `flashtex_math_layout::MathList` — as opposed to a
//! `MathList` I would type in by hand for this test.
//!
//! A repo-wide grep for `flashtex_math_layout` / `MathList` turns up:
//!
//! * `crates/math-layout/src/fixtures.rs` — a `pub mod fixtures` with 23
//!   named functions (`x_squared`, `stacked_fraction`, `sum_limits`, ...)
//!   each building a real `MathList` via the crate's own public `Atom`
//!   constructors, plus `fixtures::all()` returning every one of them paired
//!   with the LaTeX source it models. This is genuinely load-bearing
//!   production code in math-layout, not test-only scaffolding invented for
//!   this crate: it backs that crate's own golden tests
//!   (`crates/math-layout/tests/golden.rs`), its `dump` and `emit_runs`
//!   examples, and the oracle comparison documented in
//!   `crates/math-layout/docs/comparison.md`.
//! * `crates/font-engine/src/adapters/math.rs` — implements
//!   `flashtex_math_layout::metrics::MathFontMetrics` from an OpenType MATH
//!   face. It supplies font *metrics* to the layout engine; it does not
//!   build or emit a `MathList` at all, so it is not a producer of the
//!   struct this crate consumes.
//! * `crates/compiler/src/math.rs` — the compiler's real `$...$`/`\[...\]`
//!   math parser, wired into `crates/compiler/src/parser.rs` and
//!   `incremental.rs`. This is the actual end-user-facing math pipeline.
//!   But it defines and returns its own `MathList`/`MathAtom`/`Nucleus`
//!   types local to `crates/compiler` (`Symbol(String) | Fraction { .. } |
//!   Radical(..)`, scripts stored directly on `MathAtom`) — a smaller,
//!   independent model that has never been connected to
//!   `flashtex-math-layout`. It is not interchangeable with, and does not
//!   produce, `flashtex_math_layout::MathList`. (It is also a different
//!   crate than the one this agent may consume, so it could not be added as
//!   a dependency here even if the type matched.)
//!
//! **Conclusion, stated plainly:** no application or compiler path in this
//! repository currently produces a `flashtex_math_layout::MathList` for an
//! end user. The one real, non-hand-invented producer of that exact type is
//! `flashtex_math_layout::fixtures::all()`. That is what this file exercises
//! — every one of its 23 expressions, unmodified, run through
//! `MathAccessibility::describe`. If a real compiler-to-accessibility wiring
//! is added in a later revision, this file's job is to be replaced by one
//! that calls it directly.

use flashtex_math_accessibility::MathAccessibility;
use flashtex_math_layout::fixtures;

/// Every one of math-layout's own fixture expressions, run through the
/// describer, must produce a `Description` without hitting a bound: they are
/// small, hand-authored regression cases for the layout engine, nowhere near
/// [`flashtex_math_accessibility::DEFAULT_MAX_DEPTH`] or
/// [`flashtex_math_accessibility::DEFAULT_MAX_NODES`].
#[test]
fn every_math_layout_fixture_describes_without_error() {
    let corpus = fixtures::all();
    assert_eq!(
        corpus.len(),
        23,
        "math-layout's fixture corpus changed size upstream (was 23); re-run \
         the FT-038 measurement in limitation_measurement.rs against the new set"
    );
    let describer = MathAccessibility::new();
    for (name, list) in &corpus {
        let desc = describer
            .describe(list)
            .unwrap_or_else(|e| panic!("fixture {name:?} exceeded a bound: {e}"));
        // The describer must always emit at least the root-level nodes, and
        // MathML must always be a well-formed single document.
        assert!(
            desc.mathml
                .starts_with("<math xmlns=\"http://www.w3.org/1998/Math/MathML\">"),
            "fixture {name:?} produced malformed MathML root: {}",
            desc.mathml
        );
        assert!(
            desc.mathml.ends_with("</math>"),
            "fixture {name:?} produced malformed MathML root: {}",
            desc.mathml
        );
    }
}

/// Spot-checks on a few individual fixtures, so a regression in one specific
/// expression's reading is caught here rather than only in an aggregate
/// count. These are the *real* producer's own values, not hand-built ones.
#[test]
fn stacked_fraction_fixture_reads_as_nested_fractions() {
    let corpus = fixtures::all();
    let (_, list) = corpus
        .iter()
        .find(|(name, _)| *name == "\\frac{\\frac{a}{b}}{c}")
        .expect("stacked_fraction fixture present");
    let desc = MathAccessibility::new().describe(list).unwrap();
    assert_eq!(
        desc.readable,
        "start fraction, start fraction, a, over, b, end fraction, over, c, end fraction"
    );
    assert!(desc.is_fully_supported());
}

#[test]
fn lim_fixture_reads_text_operator_and_arrow_symbol() {
    let corpus = fixtures::all();
    let (_, list) = corpus
        .iter()
        .find(|(name, _)| *name == "\\lim_{x\\to 0}\\frac{\\sin x}{x}")
        .expect("lim_sin_x_over_x fixture present");
    let desc = MathAccessibility::new().describe(list).unwrap();
    assert!(desc.readable.contains("lim subscript x right arrow 0"));
    assert!(desc.readable.contains("sin x"));
    assert!(desc.is_fully_supported());
}

#[test]
fn widehat_fixture_uses_the_combining_circumflex_accent_name() {
    let corpus = fixtures::all();
    let (_, list) = corpus
        .iter()
        .find(|(name, _)| *name == "\\widehat{xyz}")
        .expect("widehat_xyz fixture present");
    let desc = MathAccessibility::new().describe(list).unwrap();
    assert_eq!(desc.readable, "x y z with hat accent");
    assert!(desc.is_fully_supported());
}
