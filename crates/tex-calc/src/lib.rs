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
}
