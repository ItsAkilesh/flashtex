//! Tokenizer.
//!
//! Produces tokens carrying exact UTF-8 byte spans into the input. Iteration is
//! over `char_indices`, so every span boundary is a real character boundary; no
//! character count is ever used where a byte offset is required.
//!
//! This is not a full TeX tokenizer. Category codes are fixed, not mutable, and
//! the recognised set is limited to what the documented subset needs. See the
//! crate README for the honest boundary.

use crate::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    /// A run of ordinary printable characters with no internal whitespace.
    Word(String),
    /// Whitespace that does not contain a blank line.
    Space,
    /// A blank line: paragraph separator.
    ParBreak,
    /// `\name` — a control word.
    Command(String),
    /// `\\` — explicit line break.
    LineBreak,
    LBrace,
    RBrace,
    /// `%` to end of line, retained so spans stay faithful to the source.
    Comment,
    /// `$`, recognised only so it can be reported as unsupported.
    MathShift,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

fn is_special(c: char) -> bool {
    matches!(c, '\\' | '{' | '}' | '%' | '$')
}

pub fn tokenize(text: &str) -> Vec<Token> {
    let bytes = text.as_bytes();
    let mut tokens = Vec::new();
    let mut it = text.char_indices().peekable();

    while let Some(&(i, c)) = it.peek() {
        if c.is_whitespace() {
            let start = i;
            let mut newlines = 0;
            let mut end = i;
            while let Some(&(j, w)) = it.peek() {
                if !w.is_whitespace() {
                    break;
                }
                if w == '\n' {
                    newlines += 1;
                }
                end = j + w.len_utf8();
                it.next();
            }
            tokens.push(Token {
                kind: if newlines >= 2 { TokenKind::ParBreak } else { TokenKind::Space },
                span: Span::new(start, end),
            });
            continue;
        }

        match c {
            '\\' => {
                let start = i;
                it.next();
                match it.peek() {
                    // `\\` is a line break, not a command named "\".
                    Some(&(j, '\\')) => {
                        it.next();
                        tokens.push(Token {
                            kind: TokenKind::LineBreak,
                            span: Span::new(start, j + 1),
                        });
                    }
                    Some(&(_, ch)) if ch.is_alphabetic() => {
                        let mut name = String::new();
                        let mut end = start + 1;
                        while let Some(&(j, ch)) = it.peek() {
                            if !ch.is_alphabetic() {
                                break;
                            }
                            name.push(ch);
                            end = j + ch.len_utf8();
                            it.next();
                        }
                        tokens.push(Token {
                            kind: TokenKind::Command(name),
                            span: Span::new(start, end),
                        });
                    }
                    // A control symbol such as `\%`: treat as escaped literal.
                    Some(&(j, ch)) => {
                        it.next();
                        tokens.push(Token {
                            kind: TokenKind::Word(ch.to_string()),
                            span: Span::new(start, j + ch.len_utf8()),
                        });
                    }
                    None => {
                        tokens.push(Token {
                            kind: TokenKind::Command(String::new()),
                            span: Span::new(start, bytes.len()),
                        });
                    }
                }
            }
            '{' | '}' | '$' => {
                it.next();
                let kind = match c {
                    '{' => TokenKind::LBrace,
                    '}' => TokenKind::RBrace,
                    _ => TokenKind::MathShift,
                };
                tokens.push(Token { kind, span: Span::new(i, i + c.len_utf8()) });
            }
            '%' => {
                let start = i;
                let mut end = i + 1;
                it.next();
                while let Some(&(j, ch)) = it.peek() {
                    if ch == '\n' {
                        break;
                    }
                    end = j + ch.len_utf8();
                    it.next();
                }
                tokens.push(Token { kind: TokenKind::Comment, span: Span::new(start, end) });
            }
            _ => {
                let start = i;
                let mut word = String::new();
                let mut end = i;
                while let Some(&(j, ch)) = it.peek() {
                    if ch.is_whitespace() || is_special(ch) {
                        break;
                    }
                    word.push(ch);
                    end = j + ch.len_utf8();
                    it.next();
                }
                tokens.push(Token { kind: TokenKind::Word(word), span: Span::new(start, end) });
            }
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_are_utf8_bytes_not_char_counts() {
        let text = "héllo wörld";
        let toks = tokenize(text);
        let words: Vec<_> = toks
            .iter()
            .filter(|t| matches!(t.kind, TokenKind::Word(_)))
            .collect();
        assert_eq!(words.len(), 2);
        // "héllo" is 6 bytes because é is two bytes.
        assert_eq!(words[0].span, Span::new(0, 6));
        assert_eq!(&text[words[0].span.start..words[0].span.end], "héllo");
        assert_eq!(&text[words[1].span.start..words[1].span.end], "wörld");
    }

    #[test]
    fn paragraph_break_needs_a_blank_line() {
        let toks = tokenize("a\nb\n\nc");
        let kinds: Vec<_> = toks.iter().map(|t| t.kind.clone()).collect();
        assert!(kinds.contains(&TokenKind::Space));
        assert!(kinds.contains(&TokenKind::ParBreak));
    }

    #[test]
    fn command_and_linebreak_are_distinguished() {
        let toks = tokenize("\\section \\\\");
        assert_eq!(toks[0].kind, TokenKind::Command("section".into()));
        assert_eq!(toks[2].kind, TokenKind::LineBreak);
    }

    // ── math shift ────────────────────────────────────────────────────────────

    #[test]
    fn dollar_produces_math_shift() {
        let toks = tokenize("$x$");
        assert!(
            toks.iter().any(|t| t.kind == TokenKind::MathShift),
            "$ must produce a MathShift token"
        );
    }

    // ── control symbols ───────────────────────────────────────────────────────

    #[test]
    fn escaped_percent_becomes_literal_word() {
        // \% is a control symbol (escaped percent) — treated as the literal '%' word.
        let toks = tokenize("\\%");
        assert_eq!(toks.len(), 1);
        assert_eq!(toks[0].kind, TokenKind::Word("%".into()));
    }

    #[test]
    fn escaped_dollar_becomes_literal_word() {
        let toks = tokenize("\\$");
        assert_eq!(toks.len(), 1);
        assert_eq!(toks[0].kind, TokenKind::Word("$".into()));
    }

    // ── comments ──────────────────────────────────────────────────────────────

    #[test]
    fn percent_starts_comment_to_end_of_line() {
        let toks = tokenize("word % this is a comment\nnext");
        let kinds: Vec<_> = toks.iter().map(|t| t.kind.clone()).collect();
        assert!(kinds.contains(&TokenKind::Comment), "% must produce a Comment token");
        // Words before and after the comment must be present.
        let words: Vec<_> = toks
            .iter()
            .filter_map(|t| if let TokenKind::Word(w) = &t.kind { Some(w.as_str()) } else { None })
            .collect();
        assert!(words.contains(&"word"), "word before comment must be tokenized");
        assert!(words.contains(&"next"), "word after comment's newline must be tokenized");
    }

    #[test]
    fn comment_span_does_not_include_trailing_newline() {
        let text = "% comment\nnext";
        let toks = tokenize(text);
        let comment = toks.iter().find(|t| t.kind == TokenKind::Comment).unwrap();
        // The newline following the comment must not be inside the Comment span.
        assert_eq!(&text[comment.span.start..comment.span.end], "% comment",
            "Comment span must cover exactly the % through the last non-newline char");
    }

    // ── braces ────────────────────────────────────────────────────────────────

    #[test]
    fn braces_are_individual_tokens() {
        let toks = tokenize("{hello}");
        assert_eq!(toks[0].kind, TokenKind::LBrace);
        assert_eq!(toks[2].kind, TokenKind::RBrace);
    }

    // ── whitespace varieties ──────────────────────────────────────────────────

    #[test]
    fn single_newline_is_space_not_par_break() {
        let toks = tokenize("a\nb");
        let kinds: Vec<_> = toks.iter().map(|t| t.kind.clone()).collect();
        assert!(kinds.contains(&TokenKind::Space));
        assert!(!kinds.contains(&TokenKind::ParBreak),
            "a single newline must not produce a ParBreak");
    }

    #[test]
    fn three_newlines_produce_par_break() {
        // ≥2 newlines in a single whitespace run → ParBreak.
        let toks = tokenize("a\n\n\nb");
        let kinds: Vec<_> = toks.iter().map(|t| t.kind.clone()).collect();
        assert!(kinds.contains(&TokenKind::ParBreak));
    }

    // ── edge cases ────────────────────────────────────────────────────────────

    #[test]
    fn empty_input_produces_no_tokens() {
        let toks = tokenize("");
        assert!(toks.is_empty(), "empty input must produce no tokens");
    }

    #[test]
    fn backslash_at_eof_produces_empty_command() {
        // A bare \ at end-of-input — no following character. The lexer
        // produces a Command("") token rather than panicking.
        let toks = tokenize("\\");
        assert_eq!(toks.len(), 1);
        assert_eq!(toks[0].kind, TokenKind::Command("".into()));
    }

    #[test]
    fn unicode_word_span_is_byte_not_char_offset() {
        // A multi-byte word must have start/end in bytes, not characters.
        let text = "αβγ";              // 6 bytes (3 × 2-byte Greek letters)
        let toks = tokenize(text);
        let word = toks.iter().find(|t| matches!(t.kind, TokenKind::Word(_))).unwrap();
        assert_eq!(word.span.end - word.span.start, text.len(),
            "span width must equal byte length of the word");
        assert_eq!(&text[word.span.start..word.span.end], "αβγ");
    }

    #[test]
    fn command_name_span_starts_at_backslash() {
        let text = "\\section";
        let toks = tokenize(text);
        assert_eq!(toks[0].kind, TokenKind::Command("section".into()));
        // The span must cover from the \ through the last letter.
        assert_eq!(toks[0].span.start, 0);
        assert_eq!(toks[0].span.end, text.len());
    }

    #[test]
    fn linebreak_token_span_covers_both_backslashes() {
        let text = "\\\\";
        let toks = tokenize(text);
        assert_eq!(toks[0].kind, TokenKind::LineBreak);
        assert_eq!(toks[0].span.start, 0);
        assert_eq!(toks[0].span.end, 2, "LineBreak span must cover both backslashes");
    }

    #[test]
    fn all_span_ranges_are_valid_utf8_slice_boundaries() {
        let text = "héllo \\section{wörld} % comm\n\\\\";
        let toks = tokenize(text);
        for tok in &toks {
            // If this panics the span is not on a UTF-8 boundary.
            let _ = &text[tok.span.start..tok.span.end];
        }
    }
}
