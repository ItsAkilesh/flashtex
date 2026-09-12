//! A small Unicode-safe lexer for TeX-flavored dimension expressions.
//!
//! Operates on `char`s (never raw bytes), so multi-byte UTF-8 in an
//! identifier, in whitespace, or in garbage input can never split a
//! character and panic. Every failure path returns [`CalcError::Parse`]
//! with the byte offset of the offending character.

use crate::sp::CalcError;

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    /// An exact decimal number, `numerator / denominator` (`denominator > 0`).
    Number(i128, i128),
    /// A bare (non-backslash) identifier: a candidate unit keyword, e.g. `pt`
    /// or an unsupported one like `em`.
    Ident(String),
    /// `\` followed by one or more Unicode alphabetic characters.
    Name(String),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semi,
    Eof,
}

/// One token together with the byte offset it started at (for error
/// reporting).
#[derive(Clone, Debug, PartialEq)]
pub struct Spanned {
    pub tok: Tok,
    pub at: usize,
}

pub fn lex(source: &str) -> Result<Vec<Spanned>, CalcError> {
    let chars: Vec<(usize, char)> = source.char_indices().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    let end_at = source.len();

    while i < chars.len() {
        let (at, c) = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        match c {
            '+' => {
                out.push(Spanned { tok: Tok::Plus, at });
                i += 1;
            }
            '-' => {
                out.push(Spanned {
                    tok: Tok::Minus,
                    at,
                });
                i += 1;
            }
            '*' => {
                out.push(Spanned { tok: Tok::Star, at });
                i += 1;
            }
            '/' => {
                out.push(Spanned {
                    tok: Tok::Slash,
                    at,
                });
                i += 1;
            }
            '(' => {
                out.push(Spanned {
                    tok: Tok::LParen,
                    at,
                });
                i += 1;
            }
            ')' => {
                out.push(Spanned {
                    tok: Tok::RParen,
                    at,
                });
                i += 1;
            }
            '{' => {
                out.push(Spanned {
                    tok: Tok::LBrace,
                    at,
                });
                i += 1;
            }
            '}' => {
                out.push(Spanned {
                    tok: Tok::RBrace,
                    at,
                });
                i += 1;
            }
            ';' => {
                out.push(Spanned { tok: Tok::Semi, at });
                i += 1;
            }
            '\\' => {
                let start = i;
                i += 1;
                let mut name = String::new();
                while i < chars.len() && chars[i].1.is_alphabetic() {
                    name.push(chars[i].1);
                    i += 1;
                }
                if name.is_empty() {
                    let bad_at = chars.get(i).map(|(p, _)| *p).unwrap_or(end_at);
                    return Err(CalcError::Parse {
                        message: "`\\` must be followed by a letter to name a length".to_string(),
                        at: bad_at,
                    });
                }
                out.push(Spanned {
                    tok: Tok::Name(name),
                    at: chars[start].0,
                });
            }
            c if c.is_ascii_digit() || c == '.' => {
                let start = i;
                let mut saw_digit = false;
                let mut saw_dot = false;
                while i < chars.len() {
                    let ch = chars[i].1;
                    if ch.is_ascii_digit() {
                        saw_digit = true;
                        i += 1;
                    } else if ch == '.' && !saw_dot {
                        saw_dot = true;
                        i += 1;
                    } else {
                        break;
                    }
                }
                if !saw_digit {
                    return Err(CalcError::Parse {
                        message: "a number needs at least one digit".to_string(),
                        at: chars[start].0,
                    });
                }
                let text: String = chars[start..i].iter().map(|(_, c)| *c).collect();
                let (numerator, denominator) = parse_decimal(&text, chars[start].0)?;
                out.push(Spanned {
                    tok: Tok::Number(numerator, denominator),
                    at: chars[start].0,
                });
            }
            c if c.is_alphabetic() => {
                let start = i;
                let mut ident = String::new();
                while i < chars.len() && chars[i].1.is_alphabetic() {
                    ident.push(chars[i].1);
                    i += 1;
                }
                out.push(Spanned {
                    tok: Tok::Ident(ident),
                    at: chars[start].0,
                });
            }
            other => {
                return Err(CalcError::Parse {
                    message: format!("unexpected character `{other}`"),
                    at,
                });
            }
        }
    }
    out.push(Spanned {
        tok: Tok::Eof,
        at: end_at,
    });
    Ok(out)
}

/// Parse an already-validated digit/dot run (`saw_digit` guaranteed) into an
/// exact `numerator/denominator` pair, e.g. `"10.5" -> (105, 10)`,
/// `".5" -> (5, 10)`, `"10." -> (10, 1)`.
fn parse_decimal(text: &str, at: usize) -> Result<(i128, i128), CalcError> {
    let (int_part, frac_part) = match text.split_once('.') {
        Some((i, f)) => (i, f),
        None => (text, ""),
    };
    let denominator: i128 = 10i128.pow(frac_part.len() as u32);
    let digits: String = format!("{int_part}{frac_part}");
    let digits = if digits.is_empty() { "0" } else { &digits };
    let numerator: i128 = digits.parse().map_err(|_| CalcError::Parse {
        message: format!("number `{text}` is too large"),
        at,
    })?;
    Ok((numerator, denominator))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toks(s: &str) -> Vec<Tok> {
        lex(s).unwrap().into_iter().map(|s| s.tok).collect()
    }

    #[test]
    fn lexes_dimension_literal() {
        assert_eq!(
            toks("3.5pt"),
            vec![Tok::Number(35, 10), Tok::Ident("pt".into()), Tok::Eof]
        );
    }

    #[test]
    fn lexes_name() {
        assert_eq!(toks("\\myLen"), vec![Tok::Name("myLen".into()), Tok::Eof]);
    }

    #[test]
    fn lexes_unicode_name() {
        assert_eq!(toks("\\Länge"), vec![Tok::Name("Länge".into()), Tok::Eof]);
    }

    #[test]
    fn bare_backslash_is_a_typed_parse_error() {
        assert!(matches!(lex("\\"), Err(CalcError::Parse { .. })));
        assert!(matches!(lex("\\ +1pt"), Err(CalcError::Parse { .. })));
    }

    #[test]
    fn bare_dot_is_a_typed_parse_error() {
        assert!(matches!(lex("."), Err(CalcError::Parse { .. })));
    }

    #[test]
    fn unexpected_symbol_is_a_typed_parse_error() {
        assert!(matches!(lex("1pt & 2pt"), Err(CalcError::Parse { .. })));
    }

    #[test]
    fn unicode_garbage_does_not_panic() {
        // Emoji, combining marks, RTL text: none of these are ASCII, all
        // must be handled by char, never byte, indexing.
        for s in [
            "🎉pt",
            "café\\x",
            "\u{0301}\u{0301}",
            "\\עברית + 1pt",
            "١٢٣pt",
        ] {
            let _ = lex(s);
        }
    }

    #[test]
    fn fractional_number_forms() {
        assert_eq!(toks(".5"), vec![Tok::Number(5, 10), Tok::Eof]);
        assert_eq!(toks("10."), vec![Tok::Number(10, 1), Tok::Eof]);
    }
}
