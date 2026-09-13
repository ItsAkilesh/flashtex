//! TeX's tokenizer (TeXbook ch. 8, "The eyes and the mouth of TeX"):
//! turns raw source bytes into tokens, respecting the current catcode
//! table, the three lexer states (N = new line, M = mid line, S =
//! skipping blanks), `^^` notation, and comments.

use crate::catcode::{CatCode, CatCodeTable};
use crate::span::Span;
use crate::token::{Token, TokenKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// Start of line: an end-of-line character here produces `\par`.
    NewLine,
    /// Middle of a line: an end-of-line character here produces a space
    /// token (catcode 10 with a single space character), unless the
    /// line's catcode-10-equivalent trailing char was already consumed.
    MidLine,
    /// Skipping blanks: spaces (and end-of-line) are swallowed until a
    /// non-space is found. Entered right after a control word or an
    /// explicit space token.
    SkipBlanks,
}

pub struct Lexer<'a> {
    src: &'a str,
    bytes: &'a [u8],
    pos: usize,
    source_id: u32,
    state: State,
    pending_par_run: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str, source_id: u32) -> Self {
        Lexer { src, bytes: src.as_bytes(), pos: 0, source_id, state: State::NewLine, pending_par_run: 0 }
    }

    fn peek_char(&self) -> Option<(char, usize)> {
        if self.pos >= self.bytes.len() {
            return None;
        }
        let rest = &self.src[self.pos..];
        rest.chars().next().map(|c| (c, c.len_utf8()))
    }

    /// Apply TeX's `^^` notation (TeXbook p. 45): `^^X` for printable X in
    /// a restricted range, and `^^xy` for two lowercase hex digits. Returns
    /// the resulting single character and how many source bytes it
    /// consumed, if a `^^` sequence was recognized at `pos`.
    fn try_superscript_notation(&self, cat_table: &CatCodeTable, pos: usize) -> Option<(char, usize)> {
        let rest = self.src.get(pos..)?;
        let mut chars = rest.char_indices();
        let (_, c0) = chars.next()?;
        if cat_table.get(c0) != CatCode::Superscript {
            return None;
        }
        let (_i1, c1) = chars.next()?;
        if c1 != c0 {
            return None;
        }
        let (i2, c2) = chars.next()?;
        // two-hex-digit form
        if let Some((i3, c3)) = chars.clone().next() {
            if c2.is_ascii_hexdigit() && c2.is_ascii_lowercase_hexish() && c3.is_ascii_hexdigit() && c3.is_ascii_lowercase_hexish()
            {
                let byte = u8::from_str_radix(&format!("{c2}{c3}"), 16).ok()?;
                let end = i3 + c3.len_utf8();
                return Some((byte as char, end));
            }
        }
        // single-char form: char code c2 XOR 64 if < 128, else + 64
        if (c2 as u32) < 128 {
            let code = (c2 as u32) ^ 64;
            let ch = char::from_u32(code)?;
            let end = i2 + c2.len_utf8();
            return Some((ch, end));
        }
        None
    }

    fn skip_comment_to_eol(&mut self) {
        while let Some((c, len)) = self.peek_char() {
            if c == '\n' {
                break;
            }
            self.pos += len;
        }
    }

    /// Read the rest of a control sequence name after the escape char has
    /// been consumed. Per TeXbook: a control word is a maximal run of
    /// catcode-11 (letter) characters; a control symbol is exactly one
    /// non-letter character (which may itself be a space, ending as a
    /// control-symbol named " "). An empty following (immediate EOL) is
    /// the "null control sequence" `\csname` builds sometimes; treated
    /// as an empty-name control word.
    fn read_cs_name(&mut self, cat_table: &CatCodeTable) -> (String, State) {
        let mut name = String::new();
        match self.peek_char() {
            None => (name, State::MidLine),
            Some((c, len)) => {
                let cat = cat_table.get(c);
                if cat == CatCode::Letter {
                    while let Some((c, len)) = self.peek_char() {
                        if cat_table.get(c) == CatCode::Letter {
                            name.push(c);
                            self.pos += len;
                        } else {
                            break;
                        }
                    }
                    (name, State::SkipBlanks)
                } else {
                    name.push(c);
                    self.pos += len;
                    let next_state = if cat == CatCode::Space { State::SkipBlanks } else { State::MidLine };
                    (name, next_state)
                }
            }
        }
    }

    /// Produce the next token, given the current catcode table (owned by
    /// the caller so `\catcode` assignments made mid-stream take effect
    /// immediately, as real TeX requires).
    pub fn next_token(&mut self, cat_table: &CatCodeTable) -> Option<Token> {
        loop {
            let start = self.pos;
            // Resolve `^^` notation into an effective character + length
            // before catcode lookup, so `^^41` etc. behave like the literal
            // character for catcode purposes.
            let (ch, raw_len) = match self.peek_char() {
                None => return None,
                Some((c, len)) => {
                    if let Some((rc, rlen)) = self.try_superscript_notation(cat_table, self.pos) {
                        (rc, rlen)
                    } else {
                        (c, len)
                    }
                }
            };
            let cat = cat_table.get(ch);
            match cat {
                CatCode::Escape => {
                    self.pos += raw_len;
                    let (name, next_state) = self.read_cs_name(cat_table);
                    self.state = next_state;
                    let span = Span::new(self.source_id, start as u32, self.pos as u32);
                    return Some(Token::new(TokenKind::ControlSequence(name), span));
                }
                CatCode::EndLine => {
                    self.pos += raw_len;
                    let span = Span::new(self.source_id, start as u32, self.pos as u32);
                    let out = match self.state {
                        State::NewLine => {
                            self.state = State::NewLine;
                            Some(Token::new(TokenKind::ControlSequence("par".to_string()), span))
                        }
                        State::MidLine => {
                            self.state = State::NewLine;
                            Some(Token::new(TokenKind::Char(' ', CatCode::Space), span))
                        }
                        State::SkipBlanks => {
                            self.state = State::NewLine;
                            continue;
                        }
                    };
                    return out;
                }
                CatCode::Space => {
                    self.pos += raw_len;
                    match self.state {
                        State::MidLine => {
                            self.state = State::SkipBlanks;
                            let span = Span::new(self.source_id, start as u32, self.pos as u32);
                            return Some(Token::new(TokenKind::Char(' ', CatCode::Space), span));
                        }
                        State::NewLine | State::SkipBlanks => continue,
                    }
                }
                CatCode::Comment => {
                    self.pos += raw_len;
                    self.skip_comment_to_eol();
                    // TeX discards the rest of the line *including* its
                    // end-of-line character (TeXbook p. 47): the next line
                    // starts in state N. Leaving the `\n` for the EndLine
                    // arm would turn every line-ending `%` into `\par`.
                    if let Some(('\n', len)) = self.peek_char() {
                        self.pos += len;
                    }
                    // A comment absorbs the following end-of-line too (it
                    // is treated as if the line ended right there); we
                    // leave the actual `\n` byte for the EndLine handling
                    // above to decide (matches TeX's "M" behavior after a
                    // comment: comment eats to EOL, then EOL is processed
                    // per current state, entering SkipBlanks-like NewLine).
                    self.state = State::NewLine;
                    continue;
                }
                CatCode::Ignored => {
                    self.pos += raw_len;
                    continue;
                }
                CatCode::Invalid => {
                    self.pos += raw_len;
                    // Real TeX raises "Text line contains an invalid
                    // character"; we skip it rather than panicking, the
                    // caller surfaces this as a diagnostic upstream.
                    continue;
                }
                CatCode::Active => {
                    self.pos += raw_len;
                    self.state = State::MidLine;
                    let span = Span::new(self.source_id, start as u32, self.pos as u32);
                    return Some(Token::new(TokenKind::ActiveChar(ch), span));
                }
                other => {
                    self.pos += raw_len;
                    self.state = State::MidLine;
                    let span = Span::new(self.source_id, start as u32, self.pos as u32);
                    return Some(Token::new(TokenKind::Char(ch, other), span));
                }
            }
        }
    }

    pub fn source_id(&self) -> u32 {
        self.source_id
    }

    pub fn byte_pos(&self) -> usize {
        self.pos
    }
}

// Small local helper trait to keep the hex-digit check readable without
// pulling in extra deps; `is_ascii_hexdigit` already covers upper+lower,
// but TeX's `^^xy` form only recognizes lowercase hex digits.
trait LowercaseHexish {
    fn is_ascii_lowercase_hexish(&self) -> bool;
}
impl LowercaseHexish for char {
    fn is_ascii_lowercase_hexish(&self) -> bool {
        self.is_ascii_digit() || ('a'..='f').contains(self)
    }
}
