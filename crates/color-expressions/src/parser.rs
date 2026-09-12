//! A small, hand-written recursive-descent parser for colour expressions.
//!
//! Grammar (no whitespace anywhere — any stray space is a parse error):
//!
//! ```text
//! MixChain   := Atom { '!' Percent [ '!' Atom ] }
//! Atom       := '-' Atom | '(' MixChain ')' | Literal | Ident
//! Literal    := Ident ':' Component { ',' Component }
//! Component  := digit+ [ '.' digit+ ]   -- parsed as f64, must be 0.0..=1.0
//! Percent    := digit+                  -- parsed as u32, must be 0..=100
//! Ident      := IdentStart IdentCont*
//! IdentStart := unicode alphabetic | '_'
//! IdentCont  := unicode alphanumeric | '_' | '-'
//! ```
//!
//! `left!pct!right` mixes `pct`% of `left` with `(100-pct)`% of `right`;
//! `left!pct` (no second `!`) mixes against white. `-atom` is the
//! component-wise complement of `atom`. See [`crate::expr`] for evaluation.
//!
//! A `Literal` is a `model:components` colour spelled out in full, resolved
//! directly to a [`flashtex_vector_graphics::Color`] with no palette lookup
//! (e.g. `rgb:1,0,0`, `cmyk:0,0,0,1`, `gray:0.5`) — the same three, and
//! only the three, colour models `flashtex_vector_graphics::Color` already
//! has. `model` is any identifier immediately followed by `:`; if it is not
//! exactly `gray`, `rgb`, or `cmyk`, or the component count doesn't match
//! that model's arity, or a component isn't a `0.0..=1.0` decimal, parsing
//! fails with a typed [`crate::ColorExprError`] — never a silent
//! approximation and never a colour space this crate's dependency doesn't
//! represent.
//!
//! Boundedness: the whole input is rejected up front if longer than
//! [`crate::MAX_INPUT_LEN`] bytes. Every call to [`Parser::atom`] — which is
//! where nesting happens, via `-` and `(...)`, and where each term of a
//! `!`-chain is produced — spends one unit from a budget of
//! [`crate::MAX_DEPTH`] atoms for the whole expression, checked *before*
//! any recursion, so a hostile input (deep parens, a long `-----` chain, or
//! a long `!`-chain) fails with [`crate::ColorExprError::TooDeep`] instead
//! of recursing without limit.

use crate::error::ColorExprError;
use crate::expr::Expr;
use crate::{MAX_DEPTH, MAX_INPUT_LEN};
use flashtex_vector_graphics::Color;

struct Parser<'a> {
    src: &'a str,
    pos: usize,
    atoms_used: usize,
}

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_ident_cont(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-'
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<char> {
        self.src[self.pos..].chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    /// Spends one unit of the atom budget. Called at the top of every
    /// [`Parser::atom`] invocation, before any recursion or node
    /// construction, so the check always happens before the work it
    /// guards.
    fn spend_depth(&mut self) -> Result<(), ColorExprError> {
        if self.atoms_used >= MAX_DEPTH {
            return Err(ColorExprError::TooDeep { max: MAX_DEPTH });
        }
        self.atoms_used += 1;
        Ok(())
    }

    fn mix_chain(&mut self) -> Result<Expr, ColorExprError> {
        let mut left = self.atom()?;
        while self.peek() == Some('!') {
            self.bump();
            let pct = self.percent()?;
            let right = if self.peek() == Some('!') {
                self.bump();
                Some(Box::new(self.atom()?))
            } else {
                None
            };
            left = Expr::Mix {
                left: Box::new(left),
                pct,
                right,
            };
        }
        Ok(left)
    }

    fn atom(&mut self) -> Result<Expr, ColorExprError> {
        self.spend_depth()?;
        match self.peek() {
            None => Err(ColorExprError::UnexpectedEnd),
            Some('-') => {
                self.bump();
                let inner = self.atom()?;
                Ok(Expr::Negate(Box::new(inner)))
            }
            Some('(') => {
                self.bump();
                let inner = self.mix_chain()?;
                match self.peek() {
                    Some(')') => {
                        self.bump();
                        Ok(inner)
                    }
                    Some(found) => Err(ColorExprError::UnexpectedChar {
                        pos: self.pos,
                        found,
                    }),
                    None => Err(ColorExprError::UnexpectedEnd),
                }
            }
            Some(c) if is_ident_start(c) => {
                let start = self.pos;
                let name = self.ident_text();
                if self.peek() == Some(':') {
                    self.bump();
                    self.literal(start, name)
                } else {
                    Ok(Expr::Name(name))
                }
            }
            Some(found) => Err(ColorExprError::UnexpectedChar {
                pos: self.pos,
                found,
            }),
        }
    }

    fn ident_text(&mut self) -> String {
        let start = self.pos;
        self.bump(); // the ident-start char, already validated by the caller
        while let Some(c) = self.peek() {
            if is_ident_cont(c) {
                self.bump();
            } else {
                break;
            }
        }
        self.src[start..self.pos].to_string()
    }

    /// Parses the `Component { ',' Component }` tail of a `model:...`
    /// literal (the `model:` prefix has already been consumed) and
    /// resolves it into a [`Color`], given the model name and its start
    /// byte offset (for error reporting). Bounded the same way the rest of
    /// the grammar is: it only ever consumes bytes already covered by
    /// [`crate::MAX_INPUT_LEN`], never loops without making progress, and
    /// every failure is a typed [`ColorExprError`].
    fn literal(&mut self, model_pos: usize, model: String) -> Result<Expr, ColorExprError> {
        // Checked before parsing any components, so an unsupported model
        // name (e.g. `hsb:...`) is always reported as such, regardless of
        // what follows the colon.
        let expected = match model.as_str() {
            "gray" => 1,
            "rgb" => 3,
            "cmyk" => 4,
            _ => {
                return Err(ColorExprError::UnsupportedColorModel {
                    pos: model_pos,
                    name: model,
                });
            }
        };
        let mut components = vec![self.component()?];
        while self.peek() == Some(',') {
            self.bump();
            components.push(self.component()?);
        }
        if components.len() != expected {
            return Err(ColorExprError::InvalidComponentCount {
                model,
                expected,
                found: components.len(),
            });
        }
        let color = match model.as_str() {
            "gray" => Color::Gray(components[0]),
            "rgb" => Color::Rgb(components[0], components[1], components[2]),
            "cmyk" => Color::Cmyk(components[0], components[1], components[2], components[3]),
            _ => unreachable!("model already validated above"),
        };
        Ok(Expr::Literal(color))
    }

    /// Parses one `digit+ ('.' digit+)?` component of a `model:...` literal
    /// and checks it falls in `0.0..=1.0`.
    fn component(&mut self) -> Result<f64, ColorExprError> {
        let start = self.pos;
        let mut text = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                text.push(c);
                self.bump();
            } else {
                break;
            }
        }
        if text.is_empty() {
            return Err(match self.peek() {
                Some(found) => ColorExprError::UnexpectedChar {
                    pos: self.pos,
                    found,
                },
                None => ColorExprError::UnexpectedEnd,
            });
        }
        if self.peek() == Some('.') {
            text.push('.');
            self.bump();
            let mut frac_digits = 0usize;
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    text.push(c);
                    self.bump();
                    frac_digits += 1;
                } else {
                    break;
                }
            }
            if frac_digits == 0 {
                return Err(match self.peek() {
                    Some(found) => ColorExprError::UnexpectedChar {
                        pos: self.pos,
                        found,
                    },
                    None => ColorExprError::UnexpectedEnd,
                });
            }
        }
        let value: f64 = text
            .parse()
            .expect("text is a validated digit+('.'digit+)? pattern");
        if !(0.0..=1.0).contains(&value) {
            return Err(ColorExprError::ComponentOutOfRange { pos: start, text });
        }
        Ok(value)
    }

    fn percent(&mut self) -> Result<u8, ColorExprError> {
        let start = self.pos;
        let mut digits = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                digits.push(c);
                self.bump();
            } else {
                break;
            }
        }
        if digits.is_empty() {
            return Err(match self.peek() {
                Some(found) => ColorExprError::UnexpectedChar {
                    pos: self.pos,
                    found,
                },
                None => ColorExprError::UnexpectedEnd,
            });
        }
        match digits.parse::<u32>() {
            Ok(v) if v <= 100 => Ok(v as u8),
            _ => Err(ColorExprError::InvalidPercentage {
                pos: start,
                text: digits,
            }),
        }
    }
}

/// Parses a colour expression into an AST, without resolving any names.
pub(crate) fn parse(input: &str) -> Result<Expr, ColorExprError> {
    if input.len() > MAX_INPUT_LEN {
        return Err(ColorExprError::TooLong {
            len: input.len(),
            max: MAX_INPUT_LEN,
        });
    }
    if input.is_empty() {
        return Err(ColorExprError::Empty);
    }
    let mut p = Parser {
        src: input,
        pos: 0,
        atoms_used: 0,
    };
    let expr = p.mix_chain()?;
    if p.pos != p.src.len() {
        return Err(ColorExprError::TrailingInput { pos: p.pos });
    }
    Ok(expr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty() {
        assert_eq!(parse(""), Err(ColorExprError::Empty));
    }

    #[test]
    fn rejects_too_long() {
        let long = "a".repeat(MAX_INPUT_LEN + 1);
        assert_eq!(
            parse(&long),
            Err(ColorExprError::TooLong {
                len: long.len(),
                max: MAX_INPUT_LEN
            })
        );
    }

    #[test]
    fn rejects_deeply_nested_parens_with_typed_error() {
        let opens = "(".repeat(MAX_DEPTH + 5);
        let closes = ")".repeat(MAX_DEPTH + 5);
        let hostile = format!("{opens}red{closes}");
        assert!(
            hostile.len() <= MAX_INPUT_LEN,
            "keep this a depth attack, not a length attack"
        );
        assert_eq!(
            parse(&hostile),
            Err(ColorExprError::TooDeep { max: MAX_DEPTH })
        );
    }

    #[test]
    fn rejects_deep_negation_chain_with_typed_error() {
        let hostile = format!("{}red", "-".repeat(MAX_DEPTH + 5));
        assert_eq!(
            parse(&hostile),
            Err(ColorExprError::TooDeep { max: MAX_DEPTH })
        );
    }

    #[test]
    fn rejects_long_mix_chain_with_typed_error() {
        let mut hostile = String::from("red");
        for _ in 0..MAX_DEPTH + 5 {
            hostile.push_str("!50!red");
        }
        assert_eq!(
            parse(&hostile),
            Err(ColorExprError::TooDeep { max: MAX_DEPTH })
        );
    }

    #[test]
    fn accepts_a_reasonable_paren_depth() {
        let opens = "(".repeat(MAX_DEPTH - 2);
        let closes = ")".repeat(MAX_DEPTH - 2);
        assert!(parse(&format!("{opens}red{closes}")).is_ok());
    }

    #[test]
    fn unexpected_char_at_start_reports_byte_offset() {
        // '#' can never start an atom.
        assert_eq!(
            parse("#red"),
            Err(ColorExprError::UnexpectedChar { pos: 0, found: '#' })
        );
    }

    #[test]
    fn unexpected_char_after_paren_reports_byte_offset() {
        // Expects ')' after the grouped expression; "(red" is 4 bytes.
        assert_eq!(
            parse("(red#)"),
            Err(ColorExprError::UnexpectedChar { pos: 4, found: '#' })
        );
    }

    #[test]
    fn unexpected_char_where_second_mix_operand_expected() {
        // "red!50!" is 7 bytes; the next atom is expected right there.
        assert_eq!(
            parse("red!50!#"),
            Err(ColorExprError::UnexpectedChar { pos: 7, found: '#' })
        );
    }

    #[test]
    fn unicode_ident_parses() {
        let e = parse("café").unwrap();
        assert_eq!(e, Expr::Name("café".to_string()));
    }

    #[test]
    fn unicode_symbol_is_a_typed_error_not_a_panic() {
        // A byte offset inside a multi-byte char must not panic when sliced.
        let err = parse("🎨").unwrap_err();
        assert_eq!(
            err,
            ColorExprError::UnexpectedChar {
                pos: 0,
                found: '🎨'
            }
        );
    }

    #[test]
    fn unicode_trailing_junk_after_a_complete_expression_is_reported_by_byte_offset() {
        // "red" parses as a complete expression; the emoji is trailing
        // input, reported at its correct (3-byte) offset, not mis-sliced.
        let err = parse("red🎨").unwrap_err();
        assert_eq!(err, ColorExprError::TrailingInput { pos: 3 });
    }

    #[test]
    fn leading_whitespace_is_rejected() {
        assert_eq!(
            parse(" red"),
            Err(ColorExprError::UnexpectedChar { pos: 0, found: ' ' })
        );
    }

    #[test]
    fn interior_whitespace_is_rejected_where_an_atom_is_expected() {
        // "red!50!" is 7 bytes; a space there can't start an atom.
        assert_eq!(
            parse("red!50! blue"),
            Err(ColorExprError::UnexpectedChar { pos: 7, found: ' ' })
        );
    }

    #[test]
    fn trailing_unmatched_paren_is_trailing_input() {
        assert_eq!(parse("red)"), Err(ColorExprError::TrailingInput { pos: 3 }));
    }

    #[test]
    fn dangling_bang_is_unexpected_end() {
        assert_eq!(parse("red!"), Err(ColorExprError::UnexpectedEnd));
    }

    #[test]
    fn percentage_over_100_is_invalid() {
        assert_eq!(
            parse("red!150!blue"),
            Err(ColorExprError::InvalidPercentage {
                pos: 4,
                text: "150".to_string()
            })
        );
    }

    #[test]
    fn mix_without_second_color_parses() {
        let e = parse("red!50").unwrap();
        assert_eq!(
            e,
            Expr::Mix {
                left: Box::new(Expr::Name("red".into())),
                pct: 50,
                right: None
            }
        );
    }

    #[test]
    fn literal_rgb_parses() {
        let e = parse("rgb:1,0,0.5").unwrap();
        assert_eq!(e, Expr::Literal(Color::Rgb(1.0, 0.0, 0.5)));
    }

    #[test]
    fn literal_gray_parses() {
        let e = parse("gray:0.25").unwrap();
        assert_eq!(e, Expr::Literal(Color::Gray(0.25)));
    }

    #[test]
    fn literal_cmyk_parses() {
        let e = parse("cmyk:0,0.5,1,0.25").unwrap();
        assert_eq!(e, Expr::Literal(Color::Cmyk(0.0, 0.5, 1.0, 0.25)));
    }

    #[test]
    fn literal_integer_components_are_whole_units() {
        // No decimal point required: "1" means 1.0, not an error.
        let e = parse("rgb:1,0,1").unwrap();
        assert_eq!(e, Expr::Literal(Color::Rgb(1.0, 0.0, 1.0)));
    }

    #[test]
    fn literal_can_appear_inside_a_mix_chain_and_be_negated() {
        let e = parse("-rgb:1,0,0!50!cmyk:0,0,0,1").unwrap();
        assert_eq!(
            e,
            Expr::Mix {
                left: Box::new(Expr::Negate(Box::new(Expr::Literal(Color::Rgb(
                    1.0, 0.0, 0.0
                ))))),
                pct: 50,
                right: Some(Box::new(Expr::Literal(Color::Cmyk(0.0, 0.0, 0.0, 1.0)))),
            }
        );
    }

    #[test]
    fn unsupported_color_model_is_a_typed_error_never_black() {
        // "hsb" is a real xcolor model, but flashtex-vector-graphics::Color
        // has no HSB variant, so this must be a typed error, not an
        // invented conversion or a silent default.
        assert_eq!(
            parse("hsb:0.5,1,1"),
            Err(ColorExprError::UnsupportedColorModel {
                pos: 0,
                name: "hsb".to_string()
            })
        );
    }

    #[test]
    fn unsupported_color_model_is_reported_even_with_malformed_components() {
        // The model name is checked before its components are parsed, so
        // this is still UnsupportedColorModel, not a component error.
        assert_eq!(
            parse("hsb:not-a-number"),
            Err(ColorExprError::UnsupportedColorModel {
                pos: 0,
                name: "hsb".to_string()
            })
        );
    }

    #[test]
    fn wrong_component_count_is_a_typed_error() {
        assert_eq!(
            parse("rgb:1,0"),
            Err(ColorExprError::InvalidComponentCount {
                model: "rgb".to_string(),
                expected: 3,
                found: 2,
            })
        );
        assert_eq!(
            parse("gray:0,0"),
            Err(ColorExprError::InvalidComponentCount {
                model: "gray".to_string(),
                expected: 1,
                found: 2,
            })
        );
    }

    #[test]
    fn component_over_one_is_out_of_range() {
        // "rgb:" is 4 bytes, so the offending component starts at byte 4.
        assert_eq!(
            parse("rgb:1.5,0,0"),
            Err(ColorExprError::ComponentOutOfRange {
                pos: 4,
                text: "1.5".to_string()
            })
        );
    }

    #[test]
    fn component_missing_fraction_digits_is_unexpected_char() {
        // "rgb:1." is 6 bytes; a trailing '.' with no fractional digit
        // can't complete a component, and the next char ends the match.
        assert_eq!(
            parse("rgb:1.,0,0"),
            Err(ColorExprError::UnexpectedChar { pos: 6, found: ',' })
        );
    }

    #[test]
    fn component_non_digit_is_unexpected_char() {
        assert_eq!(
            parse("rgb:x,0,0"),
            Err(ColorExprError::UnexpectedChar { pos: 4, found: 'x' })
        );
    }

    #[test]
    fn literal_atom_spends_the_depth_budget_like_any_other_atom() {
        // A run of negated literals should hit the same TooDeep bound as a
        // run of negated names.
        let hostile = format!("{}gray:0.5", "-".repeat(MAX_DEPTH + 5));
        assert_eq!(
            parse(&hostile),
            Err(ColorExprError::TooDeep { max: MAX_DEPTH })
        );
    }

    #[test]
    fn chained_mix_is_left_associative() {
        let e = parse("red!50!blue!25!green").unwrap();
        let expected = Expr::Mix {
            left: Box::new(Expr::Mix {
                left: Box::new(Expr::Name("red".into())),
                pct: 50,
                right: Some(Box::new(Expr::Name("blue".into()))),
            }),
            pct: 25,
            right: Some(Box::new(Expr::Name("green".into()))),
        };
        assert_eq!(e, expected);
    }
}
