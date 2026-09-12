//! Recursive-descent parser.
//!
//! ```text
//! Body    := Stmt* Expr
//! Stmt    := "\setlength" "{" Name "}" "{" Expr "}" ";"
//! Expr    := Term (("+" | "-") Term)*
//! Term    := Unary (("*" | "/") Unary)*
//! Unary   := "-" Unary | Primary
//! Primary := Number [Ident] | Name | "{" Body "}" | "(" Expr ")"
//! ```
//!
//! A bare `Number` not immediately followed by a unit keyword is a
//! dimensionless [`Expr::Scalar`], legal only as a `*`/`/` operand — that
//! restriction is enforced by the evaluator, not the grammar, so the error
//! stays a single, uniform [`CalcError`] type.

use crate::ast::{Expr, Stmt};
use crate::lexer::{Spanned, Tok, lex};
use crate::sp::{CalcError, Unit};

/// Upper bound on recursive-descent nesting depth (each level of `( ... )`,
/// `{ ... }`, or a unary `-` chain counts as one level). Parsing recurses
/// through `parse_expr -> parse_term -> parse_unary -> parse_primary`, and
/// `parse_primary` calls back into `parse_expr`/`parse_body` for `(`/`{`, so
/// unbounded input nesting would otherwise recurse the Rust call stack
/// without limit; this makes "too deeply nested" a typed [`CalcError::Parse`]
/// instead of a stack overflow, mirroring [`crate::eval::MAX_RESOLUTION_DEPTH`]
/// on the evaluation side.
pub const MAX_PARSE_DEPTH: usize = 200;

pub fn parse(source: &str) -> Result<Expr, CalcError> {
    let toks = lex(source)?;
    let mut p = Parser {
        toks,
        pos: 0,
        depth: 0,
    };
    let expr = p.parse_body()?;
    p.expect_eof()?;
    Ok(expr)
}

struct Parser {
    toks: Vec<Spanned>,
    pos: usize,
    /// Current recursive-descent nesting depth; see [`MAX_PARSE_DEPTH`].
    depth: usize,
}

impl Parser {
    fn peek(&self) -> &Tok {
        &self.toks[self.pos].tok
    }

    fn at(&self) -> usize {
        self.toks[self.pos].at
    }

    fn advance(&mut self) -> Spanned {
        let t = self.toks[self.pos].clone();
        if self.pos + 1 < self.toks.len() {
            self.pos += 1;
        }
        t
    }

    fn err(&self, message: impl Into<String>) -> CalcError {
        CalcError::Parse {
            message: message.into(),
            at: self.at(),
        }
    }

    fn expect_eof(&mut self) -> Result<(), CalcError> {
        if *self.peek() == Tok::Eof {
            Ok(())
        } else {
            Err(self.err(format!("unexpected trailing token {:?}", self.peek())))
        }
    }

    fn expect(&mut self, want: &Tok, what: &str) -> Result<(), CalcError> {
        if self.peek() == want {
            self.advance();
            Ok(())
        } else {
            Err(self.err(format!("expected {what}, found {:?}", self.peek())))
        }
    }

    fn expect_name(&mut self) -> Result<String, CalcError> {
        match self.peek().clone() {
            Tok::Name(n) => {
                self.advance();
                Ok(n)
            }
            other => Err(self.err(format!("expected a `\\name`, found {other:?}"))),
        }
    }

    /// `Stmt* Expr`, used both at the top level and inside `{ ... }`.
    fn parse_body(&mut self) -> Result<Expr, CalcError> {
        let mut stmts = Vec::new();
        loop {
            if matches!(self.peek(), Tok::Name(n) if n == "setlength") {
                stmts.push(self.parse_setlength()?);
            } else {
                break;
            }
        }
        let tail = self.parse_expr()?;
        if stmts.is_empty() {
            Ok(tail)
        } else {
            Ok(Expr::Group(stmts, Box::new(tail)))
        }
    }

    fn parse_setlength(&mut self) -> Result<Stmt, CalcError> {
        self.advance(); // \setlength
        self.expect(&Tok::LBrace, "`{`")?;
        let name = self.expect_name()?;
        self.expect(&Tok::RBrace, "`}`")?;
        self.expect(&Tok::LBrace, "`{`")?;
        let expr = self.parse_expr()?;
        self.expect(&Tok::RBrace, "`}`")?;
        self.expect(&Tok::Semi, "`;` after \\setlength{...}{...}")?;
        Ok(Stmt { name, expr })
    }

    fn parse_expr(&mut self) -> Result<Expr, CalcError> {
        let mut lhs = self.parse_term()?;
        loop {
            match self.peek() {
                Tok::Plus => {
                    self.advance();
                    let rhs = self.parse_term()?;
                    lhs = Expr::Add(Box::new(lhs), Box::new(rhs));
                }
                Tok::Minus => {
                    self.advance();
                    let rhs = self.parse_term()?;
                    lhs = Expr::Sub(Box::new(lhs), Box::new(rhs));
                }
                _ => break,
            }
        }
        Ok(lhs)
    }

    fn parse_term(&mut self) -> Result<Expr, CalcError> {
        let mut lhs = self.parse_unary()?;
        loop {
            match self.peek() {
                Tok::Star => {
                    self.advance();
                    let rhs = self.parse_unary()?;
                    lhs = Expr::Mul(Box::new(lhs), Box::new(rhs));
                }
                Tok::Slash => {
                    self.advance();
                    let rhs = self.parse_unary()?;
                    lhs = Expr::Div(Box::new(lhs), Box::new(rhs));
                }
                _ => break,
            }
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Expr, CalcError> {
        self.depth += 1;
        if self.depth > MAX_PARSE_DEPTH {
            // Every recursive path back into `parse_unary` — a nested `(`,
            // a nested `{`, or a chained unary `-` — goes through here, so
            // this one check bounds all three forms of unbounded nesting.
            let err = self.err(format!(
                "expression nesting exceeds maximum depth of {MAX_PARSE_DEPTH}"
            ));
            self.depth -= 1;
            return Err(err);
        }
        let result = if *self.peek() == Tok::Minus {
            self.advance();
            self.parse_unary().map(|inner| Expr::Neg(Box::new(inner)))
        } else {
            self.parse_primary()
        };
        self.depth -= 1;
        result
    }

    fn parse_primary(&mut self) -> Result<Expr, CalcError> {
        match self.peek().clone() {
            Tok::Number(n, d) => {
                self.advance();
                if let Tok::Ident(unit_text) = self.peek().clone() {
                    self.advance();
                    let unit = Unit::parse(&unit_text)
                        .ok_or_else(|| CalcError::UnsupportedUnit(unit_text.clone()))?;
                    Ok(Expr::Dim(n, d, unit))
                } else {
                    Ok(Expr::Scalar(n, d))
                }
            }
            Tok::Name(name) => {
                self.advance();
                Ok(Expr::Name(name))
            }
            Tok::LBrace => {
                self.advance();
                let body = self.parse_body()?;
                self.expect(&Tok::RBrace, "`}`")?;
                Ok(body)
            }
            Tok::LParen => {
                self.advance();
                let inner = self.parse_expr()?;
                self.expect(&Tok::RParen, "`)`")?;
                Ok(inner)
            }
            Tok::Ident(word) => Err(self.err(format!(
                "expected a number, `\\name`, `(` or `{{`, found bare word `{word}`"
            ))),
            other => Err(self.err(format!(
                "expected a number, `\\name`, `(` or `{{`, found {other:?}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_sum() {
        assert!(parse("1pt + 2in").is_ok());
    }

    #[test]
    fn parses_setlength_group() {
        assert!(parse("\\setlength{\\x}{1pt}; \\x + 1pt").is_ok());
    }

    #[test]
    fn parses_nested_group() {
        assert!(parse("{ \\setlength{\\x}{1pt}; \\x } + 1pt").is_ok());
    }

    #[test]
    fn unsupported_unit_is_typed_at_parse_time() {
        assert_eq!(
            parse("1em").unwrap_err(),
            CalcError::UnsupportedUnit("em".into())
        );
    }

    #[test]
    fn missing_semicolon_is_a_parse_error() {
        assert!(matches!(
            parse("\\setlength{\\x}{1pt} \\x"),
            Err(CalcError::Parse { .. })
        ));
    }

    #[test]
    fn unterminated_group_is_a_parse_error() {
        assert!(matches!(parse("{ 1pt"), Err(CalcError::Parse { .. })));
    }

    #[test]
    fn trailing_garbage_is_a_parse_error() {
        assert!(matches!(parse("1pt 2pt"), Err(CalcError::Parse { .. })));
    }

    #[test]
    fn empty_input_is_a_parse_error() {
        assert!(matches!(parse(""), Err(CalcError::Parse { .. })));
    }
}
