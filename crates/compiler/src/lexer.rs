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
    /// `\verb` (or `\verb*`) through its matching delimiter: any character
    /// immediately following `\verb`/`\verb*` — no whitespace is skipped
    /// first, unlike an ordinary control word — opens the argument, and the
    /// same character closes it. `text` is the raw source between the
    /// delimiters, untouched by `%`, `\`, `$`, `{`, or `}` interpretation:
    /// this variant is produced by a dedicated character scan, not by
    /// re-entering the normal tokenizer loop. `terminated` is false when the
    /// end of the line (or of the input) was reached before a matching
    /// closing delimiter; the parser turns that into a diagnostic and
    /// recovers at end of line, matching real LaTeX's own `\verb` error.
    Verb {
        text: String,
        starred: bool,
        terminated: bool,
    },
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
/// Callers must only apply this to genuine text-mode words. `\verb` and the
/// `verbatim`/`lstlisting` environments never reach this function — their raw
/// text is captured separately (see [`TokenKind::Verb`] and
/// `parser::verbatim_display`) precisely so it is never ligature-substituted.
/// `\texttt`/`\ttfamily` text still goes through here (an accepted
/// simplification; real TeX's typewriter fonts have no ligature program
/// either, which this crate does not yet reproduce), and math is parsed
/// through an entirely separate path that never calls this function, so every
/// other [`TokenKind::Word`] reachable from ordinary paragraph text or a
/// supported command's text argument is fair game.
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
                        if name == "lstinline" {
                            // listings.sty `\lstinline[<options>]<c>...<c>`
                            // or `{...}` (`\lstinline@`, `\lst@InlineG`): the
                            // options and the code are raw text, so the whole
                            // invocation is one `Verb` token (the render
                            // pipeline re-reads the options at its span).
                            let skip_blanks = |it: &mut std::iter::Peekable<std::str::CharIndices<'_>>| {
                                while matches!(it.peek(), Some(&(_, ' ' | '\t'))) {
                                    it.next();
                                }
                            };
                            let mut look = it.clone();
                            skip_blanks(&mut look);
                            if matches!(look.peek(), Some(&(_, '['))) {
                                let mut scan = look.clone();
                                scan.next();
                                let mut closed = false;
                                while let Some(&(_, ch)) = scan.peek() {
                                    scan.next();
                                    if ch == ']' {
                                        closed = true;
                                        break;
                                    }
                                    if ch == '\n' {
                                        break;
                                    }
                                }
                                if closed {
                                    look = scan;
                                    skip_blanks(&mut look);
                                }
                            }
                            let (code, code_end, terminated) = match look.peek().copied() {
                                Some((open, '{')) => {
                                    look.next();
                                    let mut depth = 1usize;
                                    let mut close = None;
                                    while let Some(&(j, ch)) = look.peek() {
                                        if ch == '\n' {
                                            break;
                                        }
                                        look.next();
                                        match ch {
                                            '{' => depth += 1,
                                            '}' => {
                                                depth -= 1;
                                                if depth == 0 {
                                                    close = Some(j);
                                                    break;
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                    match close {
                                        Some(j) => (text[open + 1..j].to_string(), j + 1, true),
                                        None => {
                                            let stop = look.peek().map_or(text.len(), |&(j, _)| j);
                                            (text[open + 1..stop].to_string(), stop, false)
                                        }
                                    }
                                }
                                Some((open, delim)) if delim != '\n' => {
                                    look.next();
                                    let content_start = open + delim.len_utf8();
                                    let mut close = None;
                                    while let Some(&(j, ch)) = look.peek() {
                                        if ch == '\n' {
                                            break;
                                        }
                                        look.next();
                                        if ch == delim {
                                            close = Some(j);
                                            break;
                                        }
                                    }
                                    match close {
                                        Some(j) => (text[content_start..j].to_string(), j + delim.len_utf8(), true),
                                        None => {
                                            let stop = look.peek().map_or(text.len(), |&(j, _)| j);
                                            (text[content_start..stop].to_string(), stop, false)
                                        }
                                    }
                                }
                                _ => (String::new(), end, false),
                            };
                            if code_end > end {
                                it = look;
                            }
                            tokens.push(Token {
                                kind: TokenKind::Verb {
                                    text: code,
                                    starred: false,
                                    terminated,
                                },
                                span: Span::in_document(document, start, code_end.max(end)),
                            });
                            continue;
                        }
                        if name == "verb" {
                            // `\verb`/`\verb*` reads its own raw argument
                            // directly off the character stream: no blank-
                            // skipping (the very next character, even a
                            // space, is the delimiter) and no reinterpreting
                            // `%`, `\`, `$`, `{`, `}` while scanning for the
                            // matching close.
                            let starred = matches!(it.peek(), Some(&(_, '*')));
                            if starred {
                                it.next();
                            }
                            let (verb_text, verb_end, terminated) = match it.peek().copied() {
                                Some((delim_pos, delim)) if delim != '\n' => {
                                    it.next();
                                    let content_start = delim_pos + delim.len_utf8();
                                    let mut content_end = content_start;
                                    let mut closed = false;
                                    while let Some(&(j, ch)) = it.peek() {
                                        if ch == delim {
                                            it.next();
                                            closed = true;
                                            content_end = j;
                                            break;
                                        }
                                        if ch == '\n' {
                                            break;
                                        }
                                        content_end = j + ch.len_utf8();
                                        it.next();
                                    }
                                    let verb_text = text[content_start..content_end].to_string();
                                    let verb_end = if closed {
                                        content_end + delim.len_utf8()
                                    } else {
                                        content_end
                                    };
                                    (verb_text, verb_end, closed)
                                }
                                _ => (String::new(), end, false),
                            };
                            tokens.push(Token {
                                kind: TokenKind::Verb {
                                    text: verb_text,
                                    starred,
                                    terminated,
                                },
                                span: Span::in_document(document, start, verb_end),
                            });
                            continue;
                        }
                        tokens.push(Token {
                            kind: TokenKind::Command(name),
                            span: Span::in_document(document, start, end),
                        });
                        // Real TeX enters a "skip blanks" state after a control
                        // word and silently discards the whitespace that
                        // follows — `\normalfont[4 points]` and `\normalfont
                        // [4 points]` typeset identically. A blank line still
                        // starts a new paragraph exactly as it would anywhere
                        // else, so only a run with fewer than two newlines is
                        // swallowed; this does not apply to a control
                        // *symbol* like `\ ` (control space, tokenized above
                        // as an ordinary `Word`) or `\\` (line break).
                        let mut lookahead = it.clone();
                        let mut newlines = 0;
                        let mut swallowed = false;
                        while let Some(&(_, w)) = lookahead.peek() {
                            if !w.is_whitespace() {
                                break;
                            }
                            if w == '\n' {
                                newlines += 1;
                            }
                            lookahead.next();
                            swallowed = true;
                        }
                        if swallowed && newlines < 2 {
                            it = lookahead;
                        }
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
        // The single space after the control word `\section` is swallowed
        // (TeX's "skip blanks" state), so it never becomes its own token.
        let toks = tokenize("\\section \\\\");
        assert_eq!(toks[0].kind, TokenKind::Command("section".into()));
        assert_eq!(toks[1].kind, TokenKind::LineBreak);
        assert_eq!(toks.len(), 2);
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

    #[test]
    fn verb_reads_any_delimiter_and_ignores_specials_inside() {
        let toks = tokenize(r"\verb|100% \foo${}|done");
        assert_eq!(
            toks[0].kind,
            TokenKind::Verb {
                text: r"100% \foo${}".into(),
                starred: false,
                terminated: true,
            }
        );
        // The delimiter itself is excluded from the span but the rest of the
        // token stream resumes right after it, as ordinary text.
        assert_eq!(toks[1].kind, TokenKind::Word("done".into()));
    }

    #[test]
    fn lstinline_reads_options_and_a_delimited_or_braced_body() {
        let toks = tokenize(r"\lstinline[language=C]|x = {1};| and \lstinline {a {b} c}.");
        assert_eq!(
            toks[0].kind,
            TokenKind::Verb {
                text: "x = {1};".into(),
                starred: false,
                terminated: true,
            }
        );
        assert_eq!((toks[0].span.start, toks[0].span.end), (0, 32));
        let braced = toks
            .iter()
            .find_map(|t| match &t.kind {
                TokenKind::Verb { text, .. } if text.starts_with('a') => Some(text.clone()),
                _ => None,
            })
            .expect("second \\lstinline");
        assert_eq!(braced, "a {b} c");
        assert_eq!(toks.last().map(|t| t.kind.clone()), Some(TokenKind::Word(".".into())));
    }

    #[test]
    fn unterminated_lstinline_stops_at_end_of_line() {
        let toks = tokenize("\\lstinline|open\nnext");
        assert_eq!(
            toks[0].kind,
            TokenKind::Verb {
                text: "open".into(),
                starred: false,
                terminated: false,
            }
        );
    }

    #[test]
    fn verb_star_shows_and_uses_any_delimiter_with_no_space_skipped() {
        let toks = tokenize(r"\verb* a b*");
        // The character right after `\verb*` — a space — is the delimiter,
        // with no whitespace-skipping the way an ordinary control word gets.
        assert_eq!(
            toks[0].kind,
            TokenKind::Verb {
                text: "a".into(),
                starred: true,
                terminated: true,
            }
        );
    }

    #[test]
    fn unterminated_verb_recovers_at_end_of_line() {
        let toks = tokenize("\\verb|no closing delimiter\nnext line");
        assert_eq!(
            toks[0].kind,
            TokenKind::Verb {
                text: "no closing delimiter".into(),
                starred: false,
                terminated: false,
            }
        );
        // The newline is untouched and still tokenizes normally afterward.
        assert!(toks[1..]
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::Word(w) if w == "next")));
    }

    #[test]
    fn verb_span_covers_backslash_through_closing_delimiter() {
        let text = r"\verb|xy|";
        let toks = tokenize(text);
        assert_eq!(toks[0].span, Span::new(0, text.len()));
    }
}
