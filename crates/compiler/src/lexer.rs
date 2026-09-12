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

/// Applies TeX's classic text-mode input ligatures to one word's literal text.
///
/// TeX's font ligature programs combine a double backtick or a double
/// apostrophe into a curly double quote, a lone backtick or apostrophe into a
/// curly single quote (a lone apostrophe is always a *right* single quote,
/// exactly as plain typing behaves — TeX has no separate left-single-quote
/// key), three hyphens into an em dash, two hyphens into an en dash, and an
/// exclamation mark or question mark followed by a backtick into the inverted
/// exclamation or question mark. Matching is greedy and leftmost-longest,
/// which is what reproduces TeX's own left-to-right ligature building (for
/// example four hyphens give an em dash followed by a literal hyphen, not two
/// en dashes).
///
/// Callers must only apply this to genuine text-mode words. This crate has no
/// verbatim, `\texttt`, or `\ttfamily` state yet (see the README's honest
/// boundary), and math is parsed through an entirely separate path that never
/// calls this function, so every [`TokenKind::Word`] reachable from ordinary
/// paragraph text or a supported command's text argument is fair game.
pub fn apply_text_ligatures(word: &str) -> String {
    if !word
        .bytes()
        .any(|b| matches!(b, b'`' | b'\'' | b'-' | b'!' | b'?'))
    {
        return word.to_string();
    }
    let chars: Vec<char> = word.chars().collect();
    let mut out = String::with_capacity(word.len());
    let mut i = 0;
    while i < chars.len() {
        let next = chars.get(i + 1).copied();
        match (chars[i], next) {
            ('-', Some('-')) if chars.get(i + 2) == Some(&'-') => {
                out.push('\u{2014}'); // --- -> em dash
                i += 3;
            }
            ('-', Some('-')) => {
                out.push('\u{2013}'); // -- -> en dash
                i += 2;
            }
            ('`', Some('`')) => {
                out.push('\u{201C}'); // `` -> left double quote
                i += 2;
            }
            ('\'', Some('\'')) => {
                out.push('\u{201D}'); // '' -> right double quote
                i += 2;
            }
            ('!', Some('`')) => {
                out.push('\u{00A1}'); // !` -> inverted exclamation mark
                i += 2;
            }
            ('?', Some('`')) => {
                out.push('\u{00BF}'); // ?` -> inverted question mark
                i += 2;
            }
            ('`', _) => {
                out.push('\u{2018}'); // ` -> left single quote
                i += 1;
            }
            ('\'', _) => {
                out.push('\u{2019}'); // ' -> right single quote (apostrophe)
                i += 1;
            }
            (c, _) => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
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

    #[test]
    fn quote_and_dash_ligatures_convert() {
        assert_eq!(apply_text_ligatures("``quoted''"), "\u{201C}quoted\u{201D}");
        assert_eq!(apply_text_ligatures("don't"), "don\u{2019}t");
        assert_eq!(apply_text_ligatures("`tis"), "\u{2018}tis");
        assert_eq!(apply_text_ligatures("em---dash"), "em\u{2014}dash");
        assert_eq!(apply_text_ligatures("en--dash"), "en\u{2013}dash");
        assert_eq!(apply_text_ligatures("!`Hola?`"), "\u{A1}Hola\u{BF}");
    }

    #[test]
    fn ligatures_apply_inside_a_single_compound_word() {
        // No whitespace separates these from the surrounding text, so the
        // lexer already emits them as one Word token; the ligature scan must
        // still find and convert the embedded sequences.
        assert_eq!(apply_text_ligatures("turn---after"), "turn\u{2014}after");
        assert_eq!(
            apply_text_ligatures("know''---characterize"),
            "know\u{201D}\u{2014}characterize"
        );
    }

    #[test]
    fn greedy_left_to_right_matches_tex_ligature_building() {
        // Four hyphens: (--)=en, (en,-)=em, trailing hyphen is unconsumed.
        assert_eq!(apply_text_ligatures("----"), "\u{2014}-");
        // Five hyphens: em dash then en dash, exactly as TeX's own ligature
        // program reduces them pairwise left to right.
        assert_eq!(apply_text_ligatures("-----"), "\u{2014}\u{2013}");
    }

    #[test]
    fn plain_words_and_single_hyphens_are_unchanged() {
        assert_eq!(apply_text_ligatures("hello"), "hello");
        assert_eq!(apply_text_ligatures("well-known"), "well-known");
        assert_eq!(apply_text_ligatures(""), "");
    }
}
