//! FlashTeX tex-calc: a bounded TeX dimension expression evaluator.
//!
//! An original evaluator for TeX-style dimension arithmetic, built around
//! **exact integer scaled points** rather than floating point, so results
//! are reproducible bit-for-bit and match TeX's own truncating unit
//! conversions (see [`sp`] for the hand-checked conversion table).
//!
//! ```
//! use flashtex_tex_calc::{evaluate, Sp};
//!
//! // \setlength{\x}{1in}; \x + 2pt
//! let result = evaluate(r"\setlength{\x}{1in}; \x + 2pt").unwrap();
//! assert_eq!(result, Sp(4_736_286 + 2 * 65_536));
//!
//! // Exceeding TeX's max dimension is a typed error, never a panic.
//! assert!(evaluate("16384pt").is_err());
//!
//! // A cyclic definition fails with a typed error instead of looping.
//! let cyclic = evaluate(r"\setlength{\x}{\x + 1pt}; \x");
//! assert!(cyclic.is_err());
//! ```
//!
//! ## Scope
//!
//! Grammar and semantics are documented on [`parser::parse`]. In short:
//! `{ ... }` opens a scope exactly like a TeX group — `\setlength`
//! assignments inside it are invisible once the closing `}` is reached, and
//! a name defined in an inner scope shadows the same name in an outer one.
//! Every named length is resolved lazily and lazily-cyclic definitions are
//! caught by "blackholing" a thunk while it is being forced (see [`eval`]).
//!
//! ## What this crate consumes from `document-style`
//!
//! This crate does **not** invent its own floating-point length type. The
//! only type it consumes from [`flashtex_document_style::length`] is
//! [`flashtex_document_style::length::Pt`] (an `f64` TeX point, 72.27/inch),
//! used purely as the interchange type at the boundary — see
//! [`Sp::to_style_pt`] and [`Sp::try_from_style_pt`]. Internally, every
//! dimension is an exact `i64` count of scaled points; `Pt`'s `f64` is never
//! used for arithmetic inside this crate.

pub mod ast;
pub mod deps;
pub mod eval;
pub mod lexer;
pub mod parser;
pub mod sp;

pub use ast::Expr;
pub use deps::LengthTable;
pub use eval::Value;
pub use parser::parse;
pub use sp::{CalcError, MAX_DIMEN_SP, OverflowInfo, SP_PER_PT, Sp, Unit};

/// Parse and evaluate a dimension expression in one call: the crate's main
/// entry point.
pub fn evaluate(source: &str) -> Result<Sp, CalcError> {
    let expr = parser::parse(source)?;
    eval::eval(&expr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_addition() {
        assert_eq!(evaluate("1pt + 2pt").unwrap(), Sp(3 * 65536));
    }

    #[test]
    fn scoped_named_length_basic() {
        assert_eq!(
            evaluate(r"\setlength{\x}{1in}; \x + 1pt").unwrap(),
            Sp(4_736_286 + 65536)
        );
    }

    #[test]
    fn inner_scope_shadows_outer() {
        let src = r"\setlength{\x}{1pt}; { \setlength{\x}{2pt}; \x }";
        assert_eq!(evaluate(src).unwrap(), Sp(2 * 65536));
    }

    #[test]
    fn outer_length_visible_and_unaffected_by_inner_shadow() {
        let src = r"\setlength{\x}{1pt}; { \setlength{\x}{2pt}; \x } + \x";
        // inner \x (2pt) + outer \x (1pt) = 3pt: the inner assignment did not
        // mutate the outer binding.
        assert_eq!(evaluate(src).unwrap(), Sp(3 * 65536));
    }

    #[test]
    fn inner_scope_definition_does_not_leak_out() {
        let src = r"{ \setlength{\x}{1pt}; \x } + \x";
        assert_eq!(
            evaluate(src),
            Err(CalcError::UndefinedLength("x".to_string()))
        );
    }

    #[test]
    fn self_referential_length_is_a_typed_cycle_error() {
        let src = r"\setlength{\x}{\x + 1pt}; \x";
        assert_eq!(
            evaluate(src),
            Err(CalcError::CyclicLength(vec![
                "x".to_string(),
                "x".to_string()
            ]))
        );
    }

    #[test]
    fn mutually_cyclic_lengths_are_a_typed_cycle_error() {
        let src = r"\setlength{\a}{\b + 1pt}; \setlength{\b}{\a + 1pt}; \a";
        match evaluate(src) {
            Err(CalcError::CyclicLength(chain)) => {
                assert_eq!(
                    chain,
                    vec!["a".to_string(), "b".to_string(), "a".to_string()]
                );
            }
            other => panic!("expected a cyclic-length error, got {other:?}"),
        }
    }

    #[test]
    fn three_length_cycle_reports_the_full_chain() {
        let src =
            r"\setlength{\a}{\b + 1pt}; \setlength{\b}{\c + 1pt}; \setlength{\c}{\a + 1pt}; \a";
        match evaluate(src) {
            Err(CalcError::CyclicLength(chain)) => {
                assert_eq!(
                    chain,
                    vec![
                        "a".to_string(),
                        "b".to_string(),
                        "c".to_string(),
                        "a".to_string()
                    ]
                );
            }
            other => panic!("expected a cyclic-length error, got {other:?}"),
        }
    }

    #[test]
    fn long_acyclic_chain_hits_bounded_depth_not_stack_overflow() {
        // Control-sequence names are letters only (as in real TeX), so each
        // link in the chain gets a unique name built from letters, not
        // digits: \la, \lb, \lba, \lbb, ... — `name(i)` never repeats.
        fn name(i: usize) -> String {
            let mut s = String::new();
            let mut n = i;
            loop {
                s.push((b'a' + (n % 26) as u8) as char);
                n /= 26;
                if n == 0 {
                    break;
                }
                n -= 1;
            }
            s
        }
        let mut src = String::new();
        src.push_str(&format!(r"\setlength{{\{}}}{{1sp}};", name(0)));
        for i in 1..300 {
            src.push_str(&format!(
                r"\setlength{{\{}}}{{\{}+1sp}};",
                name(i),
                name(i - 1)
            ));
        }
        src.push_str(&format!(r"\{}", name(299)));
        assert_eq!(evaluate(&src), Err(CalcError::ResolutionDepthExceeded));
    }

    #[test]
    fn undefined_length_is_typed() {
        assert_eq!(
            evaluate(r"\nope + 1pt"),
            Err(CalcError::UndefinedLength("nope".to_string()))
        );
    }

    #[test]
    fn unsupported_unit_never_approximates() {
        assert_eq!(
            evaluate("1em"),
            Err(CalcError::UnsupportedUnit("em".into()))
        );
        assert_eq!(
            evaluate("1ex"),
            Err(CalcError::UnsupportedUnit("ex".into()))
        );
        assert_eq!(
            evaluate("1px"),
            Err(CalcError::UnsupportedUnit("px".into()))
        );
    }

    #[test]
    fn overflow_from_addition_is_typed() {
        match evaluate("16383pt + 16383pt") {
            Err(CalcError::Overflow(info)) => assert_eq!(info.op, "+"),
            other => panic!("expected a typed overflow, got {other:?}"),
        }
    }

    #[test]
    fn overflow_from_literal_is_typed() {
        match evaluate("99999pt") {
            Err(CalcError::Overflow(info)) => assert_eq!(info.op, "literal"),
            other => panic!("expected a typed overflow, got {other:?}"),
        }
    }

    #[test]
    fn division_by_zero_scalar_is_typed() {
        assert_eq!(evaluate("1pt / 0"), Err(CalcError::DivisionByZero));
    }

    #[test]
    fn malformed_input_never_panics() {
        for src in [
            "",
            "+",
            "\\",
            "1",
            "1pt +",
            "{",
            "}",
            "(1pt",
            "\\setlength{\\x}{1pt}",
            "\\setlength{\\x}1pt}",
            "1..5pt",
            "\0\0\0",
            "pt1",
            "𝟙𝟚𝟛pt",
        ] {
            let _ = evaluate(src);
        }
    }

    #[test]
    fn unicode_length_names_round_trip() {
        let src = "\\setlength{\\Länge}{1cm}; \\Länge";
        assert_eq!(evaluate(src).unwrap(), Sp(1_864_679));
    }

    #[test]
    fn unicode_and_emoji_never_panic() {
        for src in ["🎉pt", "1p🎉t", "café + 1pt", "\\עברית", "\u{200B}1pt"] {
            let _ = evaluate(src);
        }
    }

    #[test]
    fn dimensionless_top_level_result_is_typed_mismatch() {
        assert!(matches!(evaluate("2 + 2"), Err(CalcError::TypeMismatch(_))));
    }

    #[test]
    fn multiply_and_divide_by_scalar() {
        assert_eq!(evaluate("2 * 3pt").unwrap(), Sp(6 * 65536));
        assert_eq!(evaluate("3pt * 2").unwrap(), Sp(6 * 65536));
        assert_eq!(evaluate("6pt / 2").unwrap(), Sp(3 * 65536));
    }

    #[test]
    fn parenthesized_precedence() {
        assert_eq!(
            evaluate("(1pt + 1pt) * 2").unwrap(),
            evaluate("4pt").unwrap()
        );
    }

    // ---- Revision 3: bounded adversarial tests ----------------------------
    //
    // Every case below targets a specific bound or diagnostic this crate
    // promises never to silently approximate, exactly at and one past the
    // bound wherever "at and past" makes sense, asserting the exact
    // `CalcError` variant (and, where rev 2 added one, its diagnostic
    // payload) rather than just `is_err()`.

    #[test]
    fn expression_at_and_past_max_dimen_cap() {
        // Exactly MAX_DIMEN_SP / -MAX_DIMEN_SP succeed; one `sp` further in
        // either direction is a typed overflow, through the full `evaluate`
        // pipeline (not just `Unit::to_sp` directly, as `sp.rs` covers).
        assert_eq!(evaluate("1073741823sp").unwrap(), Sp::MAX);
        assert_eq!(evaluate("-1073741823sp").unwrap(), Sp::MIN);
        assert!(matches!(
            evaluate("1073741824sp"),
            Err(CalcError::Overflow(_))
        ));
        assert!(matches!(
            evaluate("-1073741824sp"),
            Err(CalcError::Overflow(_))
        ));
    }

    #[test]
    fn nested_parentheses_at_and_past_the_depth_bound() {
        use crate::parser::MAX_PARSE_DEPTH;
        // `N` opening parens wrapping a primary reach recursion depth `N +
        // 1` (the outermost `parse_unary` call itself is depth 1), so `N =
        // MAX_PARSE_DEPTH - 1` is exactly at the cap and `N =
        // MAX_PARSE_DEPTH` is exactly one past it.
        let at_cap = MAX_PARSE_DEPTH - 1;
        let ok_src = format!("{}1pt{}", "(".repeat(at_cap), ")".repeat(at_cap));
        assert!(evaluate(&ok_src).is_ok());

        let past_cap = MAX_PARSE_DEPTH;
        let err_src = format!("{}1pt{}", "(".repeat(past_cap), ")".repeat(past_cap));
        assert!(matches!(evaluate(&err_src), Err(CalcError::Parse { .. })));
    }

    #[test]
    fn dependency_chain_at_and_past_max_resolution_depth() {
        // Letters-only names (as in real TeX control words), same scheme as
        // `long_acyclic_chain_hits_bounded_depth_not_stack_overflow` above.
        fn name(i: usize) -> String {
            let mut s = String::new();
            let mut n = i;
            loop {
                s.push((b'a' + (n % 26) as u8) as char);
                n /= 26;
                if n == 0 {
                    break;
                }
                n -= 1;
            }
            s
        }
        // A chain of `k` names: `\l0 = 1sp`, `\l1 = \l0 + 1sp`, ...,
        // referencing the last one.
        fn chain_src(k: usize) -> String {
            let mut src = String::new();
            src.push_str(&format!(r"\setlength{{\{}}}{{1sp}};", name(0)));
            for i in 1..k {
                src.push_str(&format!(
                    r"\setlength{{\{}}}{{\{}+1sp}};",
                    name(i),
                    name(i - 1)
                ));
            }
            src.push_str(&format!(r"\{}", name(k - 1)));
            src
        }

        // The exact number of resolution-depth units consumed per chain
        // link is an internal detail of `eval::force`/`eval_expr`, not a
        // published constant, so find the exact boundary by search rather
        // than hard-coding a multiplier: `lo` is confirmed to succeed, `hi`
        // to fail, then binary search narrows to the adjacent pair where the
        // cap is crossed.
        let lo_start = 1usize;
        let hi_start = 500usize;
        assert!(evaluate(&chain_src(lo_start)).is_ok());
        assert!(matches!(
            evaluate(&chain_src(hi_start)),
            Err(CalcError::ResolutionDepthExceeded)
        ));

        let mut lo = lo_start;
        let mut hi = hi_start;
        while lo + 1 < hi {
            let mid = lo + (hi - lo) / 2;
            match evaluate(&chain_src(mid)) {
                Ok(_) => lo = mid,
                Err(CalcError::ResolutionDepthExceeded) => hi = mid,
                other => panic!("unexpected result at chain length {mid}: {other:?}"),
            }
        }
        // `lo` is exactly at the cap (the longest chain that still
        // resolves); `hi` is exactly one link past it.
        assert!(evaluate(&chain_src(lo)).is_ok());
        assert_eq!(
            evaluate(&chain_src(hi)),
            Err(CalcError::ResolutionDepthExceeded)
        );
    }

    #[test]
    fn overflow_at_each_binary_operator_is_typed_with_the_right_op_name() {
        match evaluate("-16383pt - 16383pt") {
            Err(CalcError::Overflow(info)) => assert_eq!(info.op, "-"),
            other => panic!("expected a typed overflow via `-`, got {other:?}"),
        }
        match evaluate("16383pt * 2") {
            Err(CalcError::Overflow(info)) => assert_eq!(info.op, "*"),
            other => panic!("expected a typed overflow via `*`, got {other:?}"),
        }
        match evaluate("16383pt / 0.5") {
            // Dividing by a scalar less than 1 increases magnitude, so `/`
            // can overflow too, not just `*`.
            Err(CalcError::Overflow(info)) => assert_eq!(info.op, "/"),
            other => panic!("expected a typed overflow via `/`, got {other:?}"),
        }
    }

    #[test]
    fn division_by_a_named_length_that_evaluates_to_zero_is_typed() {
        // The zero-divisor check applies after resolving a named length's
        // value, not just to a literal `0` token in the source text.
        let src = r"\setlength{\zero}{0}; 1pt / \zero";
        assert_eq!(evaluate(src), Err(CalcError::DivisionByZero));
    }

    #[test]
    fn literal_with_an_enormous_number_of_digits_is_a_typed_parse_error() {
        // Far beyond i128::MAX (~39 digits): the literal parser must return
        // a typed `Parse` error, never panic on the failed `i128` parse.
        let src = format!("{}pt", "9".repeat(200));
        assert!(matches!(evaluate(&src), Err(CalcError::Parse { .. })));
    }

    #[test]
    fn unit_suffix_unknown_empty_or_partially_matching_is_never_approximated() {
        // Wholly unknown unit.
        assert_eq!(
            evaluate("1parsec"),
            Err(CalcError::UnsupportedUnit("parsec".to_string()))
        );
        // Shares a prefix with a real unit (`pt`) but is a distinct token:
        // never silently accepted as the unit it merely starts with.
        assert_eq!(
            evaluate("1pts"),
            Err(CalcError::UnsupportedUnit("pts".to_string()))
        );
        assert_eq!(
            evaluate("1ptx"),
            Err(CalcError::UnsupportedUnit("ptx".to_string()))
        );
        // No unit suffix at all is a dimensionless scalar, never an implicit
        // `pt`.
        assert!(matches!(evaluate("5"), Err(CalcError::TypeMismatch(_))));
        // The empty string is never a valid unit at the `Unit` level either.
        assert_eq!(crate::sp::Unit::parse(""), None);
    }

    #[test]
    fn name_shadowing_itself_in_a_nested_group_is_a_self_cycle_not_the_outer_value() {
        // The inner `\setlength{\x}{...}` shadows the outer `\x` immediately
        // -- before its own right-hand side is evaluated -- so `\x` inside
        // that right-hand side resolves to the new, still-being-forced
        // inner binding (a self-cycle), never falling back to the outer
        // 1pt binding it shadows.
        let src = r"\setlength{\x}{1pt}; { \setlength{\x}{\x + 1pt}; \x }";
        assert_eq!(
            evaluate(src),
            Err(CalcError::CyclicLength(vec![
                "x".to_string(),
                "x".to_string()
            ]))
        );
    }

    // ---- Revision 3: named adversarial categories, each a typed variant ---
    //
    // `malformed_input_never_panics` and `unicode_and_emoji_never_panic`
    // above already sweep a wide variety of garbage without asserting a
    // specific variant (their job is only "never panics"). Every category
    // the assignment names explicitly also gets its own test here that
    // asserts the exact `CalcError` variant produced, not just `is_err()`.

    #[test]
    fn nul_byte_is_a_typed_parse_error_not_a_panic() {
        assert!(matches!(evaluate("\0"), Err(CalcError::Parse { .. })));
        // A NUL embedded after otherwise-valid tokens still lexes the good
        // prefix before hitting the typed error on the bad byte.
        assert!(matches!(evaluate("1pt\0"), Err(CalcError::Parse { .. })));
        assert!(matches!(
            evaluate("1pt + \0 2pt"),
            Err(CalcError::Parse { .. })
        ));
    }

    #[test]
    fn non_nfc_unicode_is_a_typed_parse_error_not_a_panic() {
        // "e" + COMBINING ACUTE ACCENT (U+0301) is the decomposed (non-NFC)
        // form of "é" -- two `char`s, not one. The combining mark is not
        // `char::is_alphabetic` (Unicode category Mn), so the lexer's
        // backslash-name loop stops after "e" and the bare combining mark
        // is then an unrecognized character: a typed `Parse` error, never a
        // panic on the multi-byte, non-normalized encoding.
        assert!(matches!(
            evaluate("\\e\u{0301}"),
            Err(CalcError::Parse { .. })
        ));
        // Same combining mark stray in the middle of an otherwise-valid
        // expression.
        assert!(matches!(
            evaluate("1pt\u{0301} + 1pt"),
            Err(CalcError::Parse { .. })
        ));
    }

    #[test]
    fn rtl_override_character_is_a_typed_parse_error_not_a_panic() {
        // U+202E RIGHT-TO-LEFT OVERRIDE (a Unicode format character, not
        // whitespace, not alphabetic, not a digit): rejected as an
        // unrecognized character wherever it appears, never silently
        // skipped and never a panic on the bidi control point.
        assert!(matches!(
            evaluate("\u{202E}1pt"),
            Err(CalcError::Parse { .. })
        ));
        assert!(matches!(
            evaluate("1\u{202E}pt"),
            Err(CalcError::Parse { .. })
        ));
        assert!(matches!(
            evaluate("\\setlength{\\x\u{202E}}{1pt}; \\x"),
            Err(CalcError::Parse { .. })
        ));
    }

    #[test]
    fn lone_operators_are_typed_parse_errors_not_panics() {
        for src in ["+", "-", "*", "/"] {
            assert!(
                matches!(evaluate(src), Err(CalcError::Parse { .. })),
                "expected a typed parse error for lone operator {src:?}"
            );
        }
    }

    #[test]
    fn absurdly_long_flat_expression_is_bounded_not_unbounded_work() {
        // 10,000 terms joined by `+` parses in one iterative loop (no
        // recursion in `parse_expr`'s own loop), but the resulting
        // left-leaning `Add` tree is exactly 10,000 levels deep, so
        // *evaluating* it recurses through `eval_expr` far past
        // `crate::eval::MAX_RESOLUTION_DEPTH` (200). This proves an
        // absurdly long flat input is typed-rejected in bounded work
        // (this test itself completes immediately) rather than either
        // hanging or blowing the Rust call stack.
        let terms = 10_000;
        let src = format!("1sp{}", "+1sp".repeat(terms - 1));
        assert_eq!(evaluate(&src), Err(CalcError::ResolutionDepthExceeded));
    }

    #[test]
    fn numeric_literal_past_i32_and_i64_range_is_typed_overflow_not_panic() {
        // Both values fit comfortably in the `i128` the lexer/parser use for
        // literal numerators (no `Parse` error from a failed integer parse,
        // unlike `literal_with_an_enormous_number_of_digits_is_a_typed_parse_error`
        // above), but each is already far past `MAX_DIMEN_SP`. `sp` is the
        // identity unit, so this exercises `Sp::from_i128`'s bounds check
        // directly on an out-of-`i32`/out-of-`i64` magnitude without any
        // intermediate multiplication, proving the `i128` comparison never
        // wraps or panics the way a native `i64` add/compare would.
        let past_i32 = (i32::MAX as i128) + 1;
        let past_i64 = (i64::MAX as i128) + 1;
        assert!(matches!(
            evaluate(&format!("{past_i32}sp")),
            Err(CalcError::Overflow(_))
        ));
        assert!(matches!(
            evaluate(&format!("{past_i64}sp")),
            Err(CalcError::Overflow(_))
        ));
        assert!(matches!(
            evaluate(&format!("-{past_i32}sp")),
            Err(CalcError::Overflow(_))
        ));
        assert!(matches!(
            evaluate(&format!("-{past_i64}sp")),
            Err(CalcError::Overflow(_))
        ));
    }

    // ---- Rev 4: unit-scaling / scalar-multiply / scalar-divide i128
    // overflow ("wraps in release, panics in debug") --------------------
    //
    // `Unit::to_sp`, `Sp::checked_mul_scalar`, and `Sp::checked_div_scalar`
    // each computed an intermediate product on plain `i128` values before
    // ever comparing the result against `MAX_DIMEN_SP`. `i128::MAX` is
    // ~1.7e38, and each site multiplies a parsed value by a unit ratio, a
    // scalar numerator, or a scalar denominator that a valid (if absurd)
    // literal can make large enough to overflow that multiplication itself
    // -- long before the final bounds check ever runs. In a debug build
    // `overflow-checks` turns that into a panic; in `--release` it silently
    // wraps modulo 2^128, so a wildly out-of-range literal can come back as
    // `Ok(Sp(0))`, `Ok(Sp(1))`, or any other in-range value that has nothing
    // to do with the input. Each case below is a real literal/expression the
    // public parser accepts, hand-picked so the i128 wraparound lands
    // in-range instead of merely landing on a different, still-out-of-range
    // number (which would still error, just for the wrong reason).

    #[test]
    fn literal_overflows_i128_during_unit_scaling_not_after() {
        // `2^112 pt`: `mag_num * un * SP_PER_PT` (un=1 for `pt`) is
        // `2^112 * 2^16 = 2^128`, which wraps to exactly 0 modulo 2^128
        // before this fix -- i.e. an astronomically large literal silently
        // became `0sp` in release instead of a typed overflow.
        let src = "5192296858534827628530496329220096pt";
        assert!(matches!(evaluate(src), Err(CalcError::Overflow(_))));
    }

    #[test]
    fn scalar_multiply_overflows_i128_before_bounds_check() {
        // `8192pt` is `Sp(2^29)`; multiplying by the scalar `2^99` is
        // `2^29 * 2^99 = 2^128`, which wraps to exactly 0 modulo 2^128
        // before this fix, instead of the typed overflow this dimension
        // truly is.
        let src = "8192pt * 633825300114114700748351602688";
        assert!(matches!(evaluate(src), Err(CalcError::Overflow(_))));
    }

    #[test]
    fn scalar_divide_overflows_i128_before_bounds_check() {
        // `8192pt / 0.15041284052594574565471120592117694464` is really
        // `8192pt` divided by a scalar just over 0.15, i.e. about 54465pt
        // -- clearly past `MAX_DIMEN_SP` and a typed overflow. Before this
        // fix, `self.0 * denominator` (2^29 * 10^38) overflowed i128 and
        // wrapped to a value that, divided by this specific numerator,
        // silently produced `Ok(Sp(1))` instead.
        let src = "8192pt / 0.15041284052594574565471120592117694464";
        assert!(matches!(evaluate(src), Err(CalcError::Overflow(_))));
    }

    #[test]
    fn near_max_dimen_literal_and_scalar_ops_still_succeed() {
        // A fix for the above must not start rejecting ordinary large-but-
        // legal dimensions and scalar arithmetic near the real MAX_DIMEN
        // boundary. `16383.99999pt` (not `...998`) is the hand-checked exact
        // boundary literal also used in `sp::tests::boundary_exactly_at_max_dimen_via_pt`.
        assert_eq!(evaluate("16383.99999pt").unwrap(), Sp::MAX);
        assert_eq!(evaluate("-16383.99999pt").unwrap(), Sp::MIN);
        assert_eq!(evaluate("8191.5pt * 2").unwrap(), Sp(16383 * 65536));
        assert_eq!(evaluate("16383pt / 1").unwrap(), Sp(16383 * 65536));
    }
}
