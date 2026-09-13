//! `flashtex-tex-expansion`: a faithful TeX/e-TeX/LaTeX macro-expansion
//! processor for FlashTeX.
//!
//! This crate implements the "front end" of TeX (TeXbook ch. 20, and the
//! `get_next`/`expand`/`macro_call`/conditional-processing sections of
//! *The TeX Program*): category codes and tokenization, the control
//! sequence table with grouping/save-stack semantics, `\def`/`\let`-style
//! macro expansion, `\expandafter`/`\noexpand`/`\csname`, TeX and e-TeX
//! conditionals, `\count`/`\dimen`/`\skip`/`\toks` registers, and a LaTeX
//! layer (`\newcommand`, `\newenvironment`, counters). It does **not**
//! typeset anything -- its output is a flat, fully expanded/executed
//! token stream with per-token source-span provenance, meant for
//! `crates/compiler`'s layout stage to consume. See `CONTRACT.md` for the
//! proposed adoption path.

mod catcode;
mod conditionals;
mod error;
mod expand;
mod lexer;
mod macro_def;
mod registers;
mod scopes;
mod span;
mod token;

pub use catcode::{CatCode, CatCodeTable};
pub use error::{Diagnostic, Limits, Severity};
pub use expand::{Engine, Mode};
pub use registers::{DefaultFontMetrics, FontMetrics, Glue};
pub use span::Span;
pub use token::{Token, TokenKind};

/// Convenience entry point: expand a whole source string with default
/// limits and font metrics, returning the content-token stream plus any
/// diagnostics collected along the way.
pub struct ExpandResult {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn expand_str(source: &str) -> ExpandResult {
    let mut engine = Engine::new(source);
    let tokens = engine.run();
    let diagnostics = engine.diagnostics().to_vec();
    ExpandResult { tokens, diagnostics }
}

/// Render a token stream back to a plain string (concatenating character
/// tokens; control sequences render as `\name `, matching how `\message`
/// would print them) -- mainly useful for tests and the oracle harness.
pub fn tokens_to_display_string(tokens: &[Token]) -> String {
    let mut out = String::new();
    for t in tokens {
        match &t.kind {
            TokenKind::Char(c, _) => out.push(*c),
            TokenKind::ControlSequence(name) => {
                out.push('\\');
                out.push_str(name);
                if name.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false) {
                    out.push(' ');
                }
            }
            TokenKind::ActiveChar(c) => out.push(*c),
            TokenKind::Param(n) => {
                out.push('#');
                out.push_str(&n.to_string());
            }
            TokenKind::Eof => {}
        }
    }
    out
}
