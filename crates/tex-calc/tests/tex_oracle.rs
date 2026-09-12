//! Real-TeX reference oracle for this crate's dimension arithmetic
//! (GitHub issue #10, "Reference TeX oracle on every machine for output
//! comparison", scoped here to `tex-calc` only).
//!
//! Every test below drives the actual `tex` binary (plain TeX, not a
//! reimplementation) on a small `.tex` snippet, decodes the exact
//! scaled-point value TeX itself computed, and compares it against
//! [`flashtex_tex_calc::evaluate`]. **Skips cleanly, never fails,** when no
//! `tex` binary is present, following the same pattern already used for
//! oracle checks elsewhere in this workspace (`crates/pdf/tests/exact.rs`'s
//! `pdflatex()` helper, `crates/font-engine/tests/truetype.rs`'s
//! font-presence checks): a short list of well-known absolute paths, and an
//! `eprintln!("skipped: ...")` + early `return` when none exist.
//!
//! That runtime `return` alone is not enough to keep a skip from being
//! *counted* as a pass, though: a `#[test]` fn that returns normally without
//! asserting anything is reported by cargo's test harness as "ok" --
//! text-identical, in the summary, to one that actually ran the comparison.
//! Every test here that needs a real `tex` is therefore also tagged
//! `#[cfg_attr(not(tex_oracle_available), ignore = "...")]`, where
//! `tex_oracle_available` is a cfg this crate's `build.rs` sets after running
//! the same binary probe at build time. On a machine with no `tex`, these
//! tests show up in the summary as `ignored`, never silently folded into
//! `passed`. (`didot_and_font_relative_units_are_rejected_not_approximated`
//! needs no `tex` binary at all and is never ignored.)
//!
//! ## Extracting TeX's *exact* internal value
//!
//! `\showthe\dimen0` prints a decimal rounded to 5 places (the famous
//! `72.26999pt` for `1in`, not `72.27pt`), which is TeX's real answer but is
//! awkward to compare bit-for-bit. The TeXbook's own grammar is more useful
//! here: wherever an `<internal integer>` is expected, an `<internal dimen>`
//! may be used instead and "is replaced by the number of sp" it contains. So
//! `\count0=\dimen0` copies the *exact* scaled-point count into a count
//! register with no rounding at all, and `\showthe\count0` prints that raw
//! integer. Every probe below uses exactly this trick.
//!
//! ## What was found (see also `coordination/daniel-calc.md`)
//!
//! Real TeX's decimal-literal-to-`sp` conversion (tex.web's `round_decimals`
//! and `scan_dimen`) first **rounds** the written fractional decimal digits to
//! the nearest 1/65536 of the *stated* unit (ties round up), and only then
//! applies the (exact, floor-based) unit-to-point ratio for units other than
//! `pt`/`sp`. `Unit::to_sp` instead computes one fully exact rational value
//! from the literal and the unit ratio and **truncates once**, toward zero,
//! at the end. The two coincide exactly whenever the unit is `sp` (an
//! integer, nothing to round) or the literal's fractional part is already an
//! exact dyadic fraction representable in 16 bits (`.5`, `.25`, `.125`, ...,
//! or no fraction at all) -- which is every hand-checked example in
//! `sp.rs`'s own tests, including the famous `1in` = `72.26999pt` quirk.
//! They can differ, typically by a handful of `sp` (well under a
//! thousandth of a point), for a literal whose fractional part is *not*
//! such a dyadic fraction, in *any* unit including plain `pt` -- see
//! [`nonzero_non_dyadic_fractions_diverge_from_tex_rounding`] below, which
//! pins the exact, reproducible divergence. The same root cause also shifts
//! the `MAX_DIMEN` overflow boundary for converted units by a similarly
//! tiny amount -- see
//! [`max_dimen_boundary_diverges_from_tex_for_a_converted_unit`].
//!
//! This is reported as a finding, not "fixed": replicating tex.web's exact
//! two-stage round-then-floor algorithm (including its own 17-significant-
//! digit input cap and ties-round-up rule) would be a deliberate rewrite of
//! this crate's central arithmetic, trading its current "one exact
//! rational, truncate once" simplicity for bit-for-bit fidelity to a fairly
//! obscure historical TeX quirk that no realistic hand-written document's
//! precision (rarely more than 2-4 decimal digits) would ever expose in a
//! MAX_DIMEN-adjacent value. That tradeoff is left to this crate's owner;
//! nothing in `src/` was changed to chase it.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use flashtex_tex_calc::{CalcError, Sp, evaluate};

/// Locate a real TeX engine. Checked absolute paths mirror
/// `crates/pdf/tests/exact.rs`'s `pdflatex()` helper; extend this list
/// rather than assuming `tex` is on `PATH`, so the check is exact about what
/// it found.
fn find_tex() -> Option<PathBuf> {
    [
        "/Library/TeX/texbin/tex",
        "/usr/local/texlive/2026/bin/universal-darwin/tex",
        "/usr/local/texlive/2025/bin/universal-darwin/tex",
        "/usr/local/texlive/2024/bin/universal-darwin/tex",
        "/usr/bin/tex",
        "/usr/local/bin/tex",
    ]
    .iter()
    .map(PathBuf::from)
    .find(|p| p.is_file())
}

static SCRATCH_COUNTER: AtomicU64 = AtomicU64::new(0);

/// One `\dimen0 = ...` probe's outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TexResult {
    /// The exact number of scaled points TeX ended up with in `\dimen0`,
    /// decoded via `\count0=\dimen0` (never TeX's rounded 5-decimal
    /// `\showthe\dimen` text).
    sp: i64,
    /// Whether TeX logged an error while getting there (typically
    /// `"Dimension too large"` or `"Arithmetic overflow"`). TeX still
    /// leaves *some* value in the register afterward -- `max_dimen` for a
    /// literal that overflowed while being scanned, or the pre-operation
    /// value for a `\multiply`/`\divide` that overflowed -- which `sp`
    /// reports either way.
    errored: bool,
}

/// Run each of `statements` (a bare plain-TeX snippet that leaves the probed
/// value sitting in `\dimen0`, e.g. `r"\dimen0=1in"` or
/// `r"\dimen0=16383pt \advance\dimen0 by 16383pt"`) in a single `tex`
/// process, appending `\count0=\dimen0 \showthe\count0` to each. Runs in a
/// scratch directory under the OS temp dir -- never inside this repo -- that
/// is removed again before returning.
fn probe(tex: &Path, statements: &[&str]) -> Vec<TexResult> {
    let dir = std::env::temp_dir().join(format!(
        "flashtex-tex-calc-oracle-{}-{}",
        std::process::id(),
        SCRATCH_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).expect("create oracle scratch dir");

    let mut src = String::new();
    for stmt in statements {
        src.push_str(stmt);
        src.push_str(" \\count0=\\dimen0 \\showthe\\count0\n");
    }
    src.push_str("\\bye\n");
    std::fs::write(dir.join("probe.tex"), &src).expect("write probe.tex");

    let output = Command::new(tex)
        .current_dir(&dir)
        .args(["-interaction=nonstopmode", "probe.tex"])
        .stdin(Stdio::null())
        .output()
        .expect("run tex");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();

    let mut results = Vec::new();
    let mut saw_error = false;
    for line in stdout.lines() {
        if line.starts_with("! ") {
            saw_error = true;
        } else if let Some(rest) = line.strip_prefix('>') {
            let text = rest.trim().trim_end_matches('.');
            let n: i64 = text
                .parse()
                .unwrap_or_else(|e| panic!("could not parse `{line}` as a count value: {e}"));
            results.push(TexResult {
                sp: n,
                errored: saw_error,
            });
            saw_error = false;
        }
    }
    let _ = std::fs::remove_dir_all(&dir);

    assert_eq!(
        results.len(),
        statements.len(),
        "expected {} `\\showthe\\count0` results from tex, got {}; full stdout:\n{stdout}",
        statements.len(),
        results.len(),
    );
    results
}

/// Extract the leading `<num><unit>` expression's unit-and-literal text (for
/// error messages) unchanged; a tiny helper so the test tables below can be a
/// single list of `(tex_statement, crate_expr)` pairs.
fn compare_ok(tex: &Path, cases: &[(&str, &str)]) {
    let statements: Vec<&str> = cases.iter().map(|(s, _)| *s).collect();
    let results = probe(tex, &statements);
    for ((stmt, expr), got) in cases.iter().zip(results) {
        assert!(
            !got.errored,
            "{stmt:?} unexpectedly logged a TeX error for what should be an in-range value"
        );
        let ours = evaluate(expr).unwrap_or_else(|e| panic!("evaluate({expr:?}): {e}"));
        assert_eq!(
            ours,
            Sp(got.sp),
            "{expr:?}: flashtex_tex_calc says {ours:?}, real tex ({stmt:?}) says {}sp",
            got.sp
        );
    }
}

#[test]
#[cfg_attr(
    not(tex_oracle_available),
    ignore = "no real tex binary found on this machine at build time (see find_tex's candidate list); reported as ignored, not passed, so a missing oracle can never be mistaken for one that ran"
)]
fn every_supported_unit_matches_real_tex_for_one_whole_unit() {
    let Some(tex) = find_tex() else {
        eprintln!("skipped: no `tex` binary found on this machine (see find_tex's candidate list)");
        return;
    };
    // `Unit` (sp.rs) supports exactly these seven; `em`/`ex` (font-relative)
    // and `dd`/`cc` (Didot units) are not in the enum at all, so they are
    // covered separately below by asserting the typed rejection, not a
    // value comparison.
    compare_ok(
        &tex,
        &[
            (r"\dimen0=1pt", "1pt"),
            (r"\dimen0=1in", "1in"),
            (r"\dimen0=1pc", "1pc"),
            (r"\dimen0=1cm", "1cm"),
            (r"\dimen0=1mm", "1mm"),
            (r"\dimen0=1bp", "1bp"),
            (r"\dimen0=1sp", "1sp"),
        ],
    );
    // The flagship quirk this crate's docs promise: 1in is 72.26999pt, not
    // 72.27pt, because TeX's own in->pt ratio (7227/100) truncates. Checked
    // against real TeX above via the exact sp count; re-asserted here by
    // name since it is the crate's headline example.
    assert_eq!(evaluate("1in").unwrap(), Sp(4_736_286));
}

#[test]
#[cfg_attr(
    not(tex_oracle_available),
    ignore = "no real tex binary found on this machine at build time (see find_tex's candidate list); reported as ignored, not passed, so a missing oracle can never be mistaken for one that ran"
)]
fn negative_and_dyadic_fractional_literals_match_real_tex_exactly() {
    let Some(tex) = find_tex() else {
        eprintln!("skipped: no `tex` binary found on this machine");
        return;
    };
    // Every fractional literal here is an exact dyadic fraction (.5, .25,
    // .75, denominators that divide 65536) in its stated unit, so TeX's
    // intermediate round-to-nearest-1/65536 step is exact -- no rounding
    // actually occurs -- and both algorithms must agree. See the module doc
    // for why a *non*-dyadic fraction (e.g. `.1`) would not be guaranteed
    // to match.
    compare_ok(
        &tex,
        &[
            (r"\dimen0=-1in", "-1in"),
            (r"\dimen0=-1pt", "-1pt"),
            (r"\dimen0=-1073741823sp", "-1073741823sp"),
            (r"\dimen0=0.5pt", "0.5pt"),
            (r"\dimen0=0.25pt", "0.25pt"),
            (r"\dimen0=0.5in", "0.5in"),
            (r"\dimen0=10.5cm", "10.5cm"),
            (r"\dimen0=-10.5cm", "-10.5cm"),
            (r"\dimen0=3.5pc", "3.5pc"),
            (r"\dimen0=0.75mm", "0.75mm"),
            (r"\dimen0=1.5bp", "1.5bp"),
        ],
    );
}

#[test]
#[cfg_attr(
    not(tex_oracle_available),
    ignore = "no real tex binary found on this machine at build time (see find_tex's candidate list); reported as ignored, not passed, so a missing oracle can never be mistaken for one that ran"
)]
fn nonzero_non_dyadic_fractions_diverge_from_tex_rounding() {
    let Some(tex) = find_tex() else {
        eprintln!("skipped: no `tex` binary found on this machine");
        return;
    };
    // `3.1pt` needs no unit-ratio conversion at all (pt is the base unit),
    // so this isolates the *first* stage alone: TeX's `round_decimals`
    // rounds the written ".1" to the nearest 1/65536pt (6554/65536pt,
    // slightly over 0.1pt, since ties/near-ties round up), giving
    // 3pt + 6554sp = 203162sp; this crate computes the fully exact
    // 31/10 pt = 203161.6sp and truncates to 203161sp. A genuine,
    // reproducible off-by-one that has nothing to do with unit conversion.
    let results = probe(&tex, &[r"\dimen0=3.1pt", r"\dimen0=3.1cm"]);

    assert!(!results[0].errored);
    assert_eq!(
        results[0].sp, 203_162,
        "sanity: real tex's own answer for 3.1pt"
    );
    assert_eq!(
        evaluate("3.1pt").unwrap(),
        Sp(203_161),
        "this crate truncates the exact 31/10 pt to 203161sp"
    );
    assert_ne!(results[0].sp, evaluate("3.1pt").unwrap().0);

    // `3.1cm` compounds the same first-stage rounding (of ".1" in
    // *centimetres* this time) with the second-stage exact cm->pt floor
    // conversion, landing 11sp apart -- still under 0.0002pt, but not a
    // match.
    assert!(!results[1].errored);
    assert_eq!(
        results[1].sp, 5_780_518,
        "sanity: real tex's own answer for 3.1cm"
    );
    assert_eq!(
        evaluate("3.1cm").unwrap(),
        Sp(5_780_507),
        "this crate truncates the exact 31/10 * 7227/254 pt to 5780507sp"
    );
    assert_ne!(results[1].sp, evaluate("3.1cm").unwrap().0);
}

#[test]
#[cfg_attr(
    not(tex_oracle_available),
    ignore = "no real tex binary found on this machine at build time (see find_tex's candidate list); reported as ignored, not passed, so a missing oracle can never be mistaken for one that ran"
)]
fn max_dimen_boundary_matches_tex_for_pt_and_sp_literals() {
    let Some(tex) = find_tex() else {
        eprintln!("skipped: no `tex` binary found on this machine");
        return;
    };
    // For `pt` and `sp`, no unit-ratio conversion happens at all, so the
    // two-stage-rounding gap above cannot shift the boundary: both this
    // crate and real TeX agree exactly that the cutoff is the literal
    // `16384pt` / `1073741824sp` (one past `MAX_DIMEN_SP`), in both
    // directions.
    let statements = [
        r"\dimen0=16383.99999pt",
        r"\dimen0=16384pt",
        r"\dimen0=-16384pt",
        r"\dimen0=1073741823sp",
        r"\dimen0=1073741824sp",
        r"\dimen0=-1073741823sp",
        r"\dimen0=-1073741824sp",
    ];
    let results = probe(&tex, &statements);

    assert!(!results[0].errored);
    assert_eq!(results[0].sp, Sp::MAX.0);
    assert_eq!(evaluate("16383.99999pt").unwrap(), Sp::MAX);

    assert!(
        results[1].errored,
        "real tex must report Dimension too large for 16384pt"
    );
    assert_eq!(
        results[1].sp,
        Sp::MAX.0,
        "tex clamps the overflowing literal to max_dimen"
    );
    assert!(matches!(evaluate("16384pt"), Err(CalcError::Overflow(_))));

    assert!(results[2].errored);
    assert_eq!(results[2].sp, Sp::MIN.0);
    assert!(matches!(evaluate("-16384pt"), Err(CalcError::Overflow(_))));

    assert!(!results[3].errored);
    assert_eq!(results[3].sp, Sp::MAX.0);
    assert_eq!(evaluate("1073741823sp").unwrap(), Sp::MAX);

    assert!(results[4].errored);
    assert_eq!(results[4].sp, Sp::MAX.0);
    assert!(matches!(
        evaluate("1073741824sp"),
        Err(CalcError::Overflow(_))
    ));

    assert!(!results[5].errored);
    assert_eq!(results[5].sp, Sp::MIN.0);
    assert_eq!(evaluate("-1073741823sp").unwrap(), Sp::MIN);

    assert!(results[6].errored);
    assert_eq!(results[6].sp, Sp::MIN.0);
    assert!(matches!(
        evaluate("-1073741824sp"),
        Err(CalcError::Overflow(_))
    ));
}

#[test]
#[cfg_attr(
    not(tex_oracle_available),
    ignore = "no real tex binary found on this machine at build time (see find_tex's candidate list); reported as ignored, not passed, so a missing oracle can never be mistaken for one that ran"
)]
fn max_dimen_boundary_diverges_from_tex_for_a_converted_unit() {
    let Some(tex) = find_tex() else {
        eprintln!("skipped: no `tex` binary found on this machine");
        return;
    };
    // Genuine boundary mismatch (see module doc): this crate's exact
    // rational threshold for `in` is 1638400/7227 = 226.70541026705...in
    // (last safe literal at 10-digit precision: 226.70541026in). Real TeX
    // instead rounds the written fraction to the nearest 1/65536 *inch*
    // before converting, so its own threshold is lower: the largest safe
    // rounded fraction is 46229/65536in (226 + 46229/65536 =
    // 226.70539855957031...in), landing the literal cutoff between
    // `226.70540618in` (still safe) and `226.70540619in` (already "Dimension
    // too large"). For every literal in `[226.70540619, 226.70541027)in`,
    // real TeX refuses the dimension outright while this crate's `evaluate`
    // returns `Ok`.
    let statements = [
        r"\dimen0=226.70540618in", // real tex: last safe value
        r"\dimen0=226.70540619in", // real tex: first "Dimension too large"
        r"\dimen0=226.70541026in", // this crate: last safe value
        r"\dimen0=226.70541027in", // this crate: first Overflow
    ];
    let results = probe(&tex, &statements);

    assert!(
        !results[0].errored,
        "real tex's own boundary: last safe value"
    );
    assert_eq!(results[0].sp, 1_073_741_768);
    assert!(
        evaluate("226.70540618in").is_ok(),
        "this crate also accepts it (not yet at its own, later threshold)"
    );

    assert!(
        results[1].errored,
        "real tex's own boundary: first Dimension-too-large"
    );
    assert_eq!(results[1].sp, Sp::MAX.0);
    // The genuine mismatch: this crate is still perfectly happy here, ~4e-6
    // inch (~0.0003pt) before its own threshold.
    assert!(
        evaluate("226.70540619in").is_ok(),
        "known divergence: this crate accepts a value real tex already refuses"
    );

    // Real tex has been in "Dimension too large" (clamped to max_dimen)
    // ever since case 1 above; it is still erroring here too, right up to
    // this crate's own (later) cutoff -- the whole
    // `[226.70540619, 226.70541027)in` range is real-tex-refuses/
    // this-crate-accepts.
    assert!(
        results[2].errored,
        "real tex is still erroring at this crate's own boundary"
    );
    assert_eq!(results[2].sp, Sp::MAX.0);
    assert_eq!(
        evaluate("226.70541026in").unwrap(),
        Sp::MAX,
        "known divergence: this crate's own last-safe value, while real tex has long refused it"
    );

    // One ulp past this crate's own cutoff, both finally agree again: real
    // tex is (still) erroring, and now so is this crate.
    assert!(results[3].errored);
    assert!(matches!(
        evaluate("226.70541027in"),
        Err(CalcError::Overflow(_))
    ));
}

#[test]
#[cfg_attr(
    not(tex_oracle_available),
    ignore = "no real tex binary found on this machine at build time (see find_tex's candidate list); reported as ignored, not passed, so a missing oracle can never be mistaken for one that ran"
)]
fn tex_advance_and_multiply_do_not_bound_check_against_max_dimen_but_this_crate_always_does() {
    let Some(tex) = find_tex() else {
        eprintln!("skipped: no `tex` binary found on this machine");
        return;
    };
    // `\advance` performs no MAX_DIMEN bound check at all: going from
    // 16383pt to 2x16383pt = 32766pt (nearly double max_dimen) logs no
    // error whatsoever. This crate's `checked_add` always enforces the
    // bound and returns a typed `Overflow` for the equivalent expression --
    // a deliberate, documented improvement over real TeX's behavior here,
    // not a bug to match.
    let results = probe(&tex, &[r"\dimen0=16383pt \advance\dimen0 by 16383pt"]);
    assert!(
        !results[0].errored,
        "real tex logs no error even though the result is far past MAX_DIMEN_SP"
    );
    assert_eq!(results[0].sp, 32_766 * 65_536);
    assert!(
        results[0].sp > flashtex_tex_calc::MAX_DIMEN_SP,
        "confirms real tex's raw register value exceeds this crate's own bound"
    );
    assert!(matches!(
        evaluate("16383pt + 16383pt"),
        Err(CalcError::Overflow(_))
    ));

    // `\multiply` is, bizarrely, *stricter* than `\advance` in real TeX: its
    // own overflow check trips at 2x16383pt already (an asymmetry internal
    // to TeX's implementation, not something this crate attempts to
    // replicate -- it applies one consistent MAX_DIMEN bound to every
    // operator). On overflow `\multiply` leaves the register at its
    // pre-multiply value rather than clamping to max_dimen.
    let results = probe(&tex, &[r"\dimen0=16383pt \multiply\dimen0 by 2"]);
    assert!(
        results[0].errored,
        "real tex's own \\multiply overflow check"
    );
    assert_eq!(
        results[0].sp,
        16_383 * 65_536,
        "tex leaves \\multiply's operand unchanged on its own overflow"
    );
    assert!(matches!(
        evaluate("16383pt * 2"),
        Err(CalcError::Overflow(_))
    ));
}

#[test]
#[cfg_attr(
    not(tex_oracle_available),
    ignore = "no real tex binary found on this machine at build time (see find_tex's candidate list); reported as ignored, not passed, so a missing oracle can never be mistaken for one that ran"
)]
fn integer_multiply_and_divide_truncation_matches_tex() {
    let Some(tex) = find_tex() else {
        eprintln!("skipped: no `tex` binary found on this machine");
        return;
    };
    // `\multiply`/`\divide` only take integer counts (unlike this crate's
    // `*`/`/`, which also accept a decimal scalar via its own grammar, not a
    // plain-TeX primitive, so a decimal scalar is not oracle-checked here).
    // Both engines truncate `/` toward zero.
    let results = probe(
        &tex,
        &[
            r"\dimen0=1pt \multiply\dimen0 by 2",
            r"\dimen0=1pt \divide\dimen0 by 3",
            r"\dimen0=1pt \divide\dimen0 by -3",
        ],
    );
    for r in &results {
        assert!(!r.errored);
    }
    assert_eq!(results[0].sp, evaluate("2 * 1pt").unwrap().0);
    assert_eq!(results[1].sp, evaluate("1pt / 3").unwrap().0);
    assert_eq!(results[2].sp, evaluate("1pt / -3").unwrap().0);
}

/// `em`/`ex` (font-relative) and `dd`/`cc` (Didot point / cicero) are not in
/// `sp.rs`'s `Unit` enum at all -- this crate rejects them outright rather
/// than approximating a font-relative length or adding two more exact
/// ratios, so there is no oracle value to compare against; only the typed
/// rejection is asserted, and it needs no `tex` binary.
///
/// For the record, real TeX (BasicTeX / TeX Live 2026, pdfTeX
/// 3.141592653-2.6-1.40.29) computes `1dd` = `70124sp` (1.07pt) and `1cc` =
/// `841489sp` (12.8401pt) -- confirmed live while building this oracle, kept
/// here as a comment rather than a live-run assertion since this crate has
/// nothing to compare them against.
#[test]
fn didot_and_font_relative_units_are_rejected_not_approximated() {
    assert_eq!(
        evaluate("1dd"),
        Err(CalcError::UnsupportedUnit("dd".into()))
    );
    assert_eq!(
        evaluate("1cc"),
        Err(CalcError::UnsupportedUnit("cc".into()))
    );
    assert_eq!(
        evaluate("1em"),
        Err(CalcError::UnsupportedUnit("em".into()))
    );
    assert_eq!(
        evaluate("1ex"),
        Err(CalcError::UnsupportedUnit("ex".into()))
    );
}
