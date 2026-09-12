//! Tokenizer.
//!
//! Produces tokens carrying exact UTF-8 byte spans into the input. Iteration is
//! over `char_indices`, so every span boundary is a real character boundary; no
//! character count is ever used where a byte offset is required.
//!
//! This is not a full TeX tokenizer. Category codes are fixed, not mutable, and
//! the recognised set is limited to what the documented subset needs. See the
//! crate README for the honest boundary.

use crate::{DocumentId, Span};

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
    /// `$`, used once for inline math and twice for display math.
    MathShift,
    /// `\[` and `\]`, the alternate display-math delimiters.
    DisplayMathOpen,
    DisplayMathClose,
    Superscript,
    Subscript,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

fn is_special(c: char) -> bool {
    matches!(c, '\\' | '{' | '}' | '%' | '$' | '^' | '_')
}

pub fn tokenize(text: &str) -> Vec<Token> {
    tokenize_document(text, DocumentId::default())
}

/// Tokenize one member of a project while retaining its document identity.
pub fn tokenize_document(text: &str, document: DocumentId) -> Vec<Token> {
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
                kind: if newlines >= 2 {
                    TokenKind::ParBreak
                } else {
                    TokenKind::Space
                },
                span: Span::in_document(document, start, end),
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
                            span: Span::in_document(document, start, j + 1),
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
                            span: Span::in_document(document, start, end),
                        });
                    }
                    Some(&(j, '[')) | Some(&(j, ']')) => {
                        let open = matches!(it.peek(), Some((_, '[')));
                        it.next();
                        tokens.push(Token {
                            kind: if open {
                                TokenKind::DisplayMathOpen
                            } else {
                                TokenKind::DisplayMathClose
                            },
                            span: Span::in_document(document, start, j + 1),
                        });
                    }
                    // A control symbol such as `\%`: treat as escaped literal.
                    Some(&(j, ch)) => {
                        it.next();
                        tokens.push(Token {
                            kind: TokenKind::Word(ch.to_string()),
                            span: Span::in_document(document, start, j + ch.len_utf8()),
                        });
                    }
                    None => {
                        tokens.push(Token {
                            kind: TokenKind::Command(String::new()),
                            span: Span::in_document(document, start, bytes.len()),
                        });
                    }
                }
            }
            '{' | '}' | '$' | '^' | '_' => {
                it.next();
                let kind = match c {
                    '{' => TokenKind::LBrace,
                    '}' => TokenKind::RBrace,
                    '$' => TokenKind::MathShift,
                    '^' => TokenKind::Superscript,
                    _ => TokenKind::Subscript,
                };
                tokens.push(Token {
                    kind,
                    span: Span::in_document(document, i, i + c.len_utf8()),
                });
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
                tokens.push(Token {
                    kind: TokenKind::Comment,
                    span: Span::in_document(document, start, end),
                });
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
                tokens.push(Token {
                    kind: TokenKind::Word(word),
                    span: Span::in_document(document, start, end),
                });
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
}
