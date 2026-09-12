//! Parser for the documented LaTeX subset.
//!
//! Honest boundary: this is a finite grammar, not TeX. There is no macro
//! expansion, no category-code mutation, no register or conditional handling,
//! and no package loading. Commands outside the supported set are reported as
//! unsupported and their argument text is still typeset where that is
//! unambiguous, so the author sees their words rather than silence.

use crate::diagnostics::Diagnostic;
use crate::lexer::{tokenize, Token, TokenKind};
use crate::Span;

use crate::math::{self, MathList};

#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    Text {
        text: String,
        span: Span,
    },
    LineBreak {
        span: Span,
    },
    Math {
        list: MathList,
        display: bool,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Paragraph(Vec<Inline>),
    Heading { level: u8, content: Vec<Inline> },
}

#[derive(Debug)]
pub struct Parsed {
    pub blocks: Vec<Block>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Commands this version actually implements.
const SUPPORTED: &[&str] = &[
    "section",
    "subsection",
    "textbf",
    "emph",
    "textit",
    "begin",
    "end",
    "par",
];

pub fn parse(text: &str) -> Parsed {
    let tokens = tokenize(text);
    let mut p = P {
        t: tokens,
        i: 0,
        diags: Vec::new(),
        brace_stack: Vec::new(),
        env_stack: Vec::new(),
    };
    let blocks = p.document();

    // Anything still open at EOF is reported with the position that opened it.
    while let Some(open) = p.brace_stack.pop() {
        p.diags.push(Diagnostic::error(
            "unmatched '{' — group never closed",
            Some(open),
            Some("treated the rest of the document as part of the group".into()),
        ));
    }
    while let Some((name, span)) = p.env_stack.pop() {
        p.diags.push(Diagnostic::error(
            format!("unterminated environment '{}' — no matching \\end", name),
            Some(span),
            Some("closed the environment at end of input".into()),
        ));
    }

    Parsed {
        blocks,
        diagnostics: p.diags,
    }
}

struct P {
    t: Vec<Token>,
    i: usize,
    diags: Vec<Diagnostic>,
    brace_stack: Vec<Span>,
    env_stack: Vec<(String, Span)>,
}

impl P {
    fn peek(&self) -> Option<&Token> {
        self.t.get(self.i)
    }

    fn document(&mut self) -> Vec<Block> {
        let mut blocks = Vec::new();
        let mut para: Vec<Inline> = Vec::new();

        while self.i < self.t.len() {
            let tok = self.t[self.i].clone();
            match tok.kind {
                TokenKind::ParBreak => {
                    self.i += 1;
                    if !para.is_empty() {
                        blocks.push(Block::Paragraph(std::mem::take(&mut para)));
                    }
                }
                TokenKind::Space | TokenKind::Comment => {
                    self.i += 1;
                }
                TokenKind::Word(w) => {
                    self.i += 1;
                    para.push(Inline::Text {
                        text: w,
                        span: tok.span,
                    });
                }
                TokenKind::LineBreak => {
                    self.i += 1;
                    para.push(Inline::LineBreak { span: tok.span });
                }
                TokenKind::LBrace => {
                    self.i += 1;
                    self.brace_stack.push(tok.span);
                }
                TokenKind::RBrace => {
                    self.i += 1;
                    if self.brace_stack.pop().is_none() {
                        self.diags.push(Diagnostic::error(
                            "unmatched '}' — no group is open here",
                            Some(tok.span),
                            Some("ignored the stray brace and continued".into()),
                        ));
                    }
                }
                TokenKind::MathShift => self.dollar_math(tok.span, &mut para),
                TokenKind::DisplayMathOpen => self.bracket_math(tok.span, &mut para),
                TokenKind::DisplayMathClose => {
                    self.i += 1;
                    self.diags.push(Diagnostic::error(
                        "stray \\] has no matching \\[",
                        Some(tok.span),
                        Some("ignored the stray display-math delimiter".into()),
                    ));
                }
                TokenKind::Superscript | TokenKind::Subscript => {
                    self.i += 1;
                    self.diags.push(Diagnostic::error(
                        "math script marker used outside math mode",
                        Some(tok.span),
                        Some("ignored the script marker and continued".into()),
                    ));
                }
                TokenKind::Command(name) => {
                    self.i += 1;
                    self.command(&name, tok.span, &mut blocks, &mut para);
                }
            }
        }

        if !para.is_empty() {
            blocks.push(Block::Paragraph(para));
        }
        blocks
    }

    fn command(&mut self, name: &str, span: Span, blocks: &mut Vec<Block>, para: &mut Vec<Inline>) {
        match name {
            "section" | "subsection" => {
                let level = if name == "section" { 1 } else { 2 };
                let (content, _) = self.required_argument(name, span);
                if !para.is_empty() {
                    blocks.push(Block::Paragraph(std::mem::take(para)));
                }
                blocks.push(Block::Heading { level, content });
            }
            // Styling is parsed and its text typeset; the visual weight is not
            // yet applied, and that limitation is stated in the README.
            "textbf" | "emph" | "textit" => {
                let (content, _) = self.required_argument(name, span);
                para.extend(content);
            }
            "begin" | "end" => {
                let (content, arg_span) = self.required_argument(name, span);
                let env: String = content
                    .iter()
                    .filter_map(|i| match i {
                        Inline::Text { text, .. } => Some(text.clone()),
                        Inline::LineBreak { .. } | Inline::Math { .. } => None,
                    })
                    .collect();
                if name == "begin" {
                    if env != "document" {
                        self.diags.push(Diagnostic::warning(
                            format!("environment '{}' is not implemented; its body is typeset as plain text", env),
                            Some(span),
                            Some("typeset the body without the environment's formatting".into()),
                        ));
                    }
                    self.env_stack.push((env, span.merge(arg_span)));
                } else {
                    match self.env_stack.pop() {
                        Some((open, _)) if open == env => {}
                        Some((open, open_span)) => {
                            self.diags.push(Diagnostic::error(
                                format!("\\end{{{}}} does not match \\begin{{{}}}", env, open),
                                Some(span),
                                Some("closed the innermost open environment".into()),
                            ));
                            let _ = open_span;
                        }
                        None => self.diags.push(Diagnostic::error(
                            format!("\\end{{{}}} with no matching \\begin", env),
                            Some(span),
                            Some("ignored the stray \\end".into()),
                        )),
                    }
                }
            }
            "par" => {
                if !para.is_empty() {
                    blocks.push(Block::Paragraph(std::mem::take(para)));
                }
            }
            other => {
                debug_assert!(!SUPPORTED.contains(&other));
                self.diags.push(Diagnostic::error(
                    format!(
                        "\\{} is not supported by this compiler version; unrestricted TeX math mode is not implemented",
                        other
                    ),
                    Some(span),
                    Some(
                        "skipped the command; any braced argument was typeset as plain text".into(),
                    ),
                ));
            }
        }
    }

    fn dollar_math(&mut self, open: Span, para: &mut Vec<Inline>) {
        self.i += 1;
        let display = matches!(self.peek().map(|t| &t.kind), Some(TokenKind::MathShift));
        if display {
            self.i += 1;
        }
        let content_start = self.i;
        let mut content_end = self.t.len();
        let mut close_end = open.end;
        let mut found = false;
        while self.i < self.t.len() {
            if self.t[self.i].kind == TokenKind::MathShift {
                let closes = !display
                    || self.t.get(self.i + 1).map(|t| &t.kind) == Some(&TokenKind::MathShift);
                if closes {
                    content_end = self.i;
                    close_end = if display {
                        self.t[self.i + 1].span.end
                    } else {
                        self.t[self.i].span.end
                    };
                    self.i += if display { 2 } else { 1 };
                    found = true;
                    break;
                }
            }
            self.i += 1;
        }
        self.finish_math(
            open,
            content_start,
            content_end,
            close_end,
            found,
            display,
            para,
        );
    }

    fn bracket_math(&mut self, open: Span, para: &mut Vec<Inline>) {
        self.i += 1;
        let content_start = self.i;
        while self.i < self.t.len() && self.t[self.i].kind != TokenKind::DisplayMathClose {
            self.i += 1;
        }
        let content_end = self.i;
        let found = self.i < self.t.len();
        let close_end = if found {
            let end = self.t[self.i].span.end;
            self.i += 1;
            end
        } else {
            open.end
        };
        self.finish_math(
            open,
            content_start,
            content_end,
            close_end,
            found,
            true,
            para,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_math(
        &mut self,
        open: Span,
        content_start: usize,
        content_end: usize,
        close_end: usize,
        found: bool,
        display: bool,
        para: &mut Vec<Inline>,
    ) {
        let raw = &self.t[content_start..content_end];
        let list = math::parse_tokens(raw, &mut self.diags);
        let end = if found {
            close_end
        } else {
            raw.last().map_or(open.end, |t| t.span.end)
        };
        if !found {
            self.diags.push(Diagnostic::error(
                if display {
                    "display math is missing its closing delimiter"
                } else {
                    "inline math is missing its closing '$'"
                },
                Some(open),
                Some("closed math mode at end of input and typeset its contents".into()),
            ));
        }
        para.push(Inline::Math {
            list,
            display,
            span: Span::new(open.start, end),
        });
    }

    /// Reads a `{...}` argument. Returns its inlines and the span covering it.
    ///
    /// A missing or empty argument is a diagnostic, never a silent default.
    fn required_argument(&mut self, cmd: &str, cmd_span: Span) -> (Vec<Inline>, Span) {
        // Skip whitespace between the command and its argument.
        while matches!(self.peek().map(|t| &t.kind), Some(TokenKind::Space)) {
            self.i += 1;
        }
        let open = match self.peek() {
            Some(t) if t.kind == TokenKind::LBrace => t.span,
            _ => {
                self.diags.push(Diagnostic::error(
                    format!("\\{} requires a braced argument", cmd),
                    Some(cmd_span),
                    Some("used an empty argument and continued".into()),
                ));
                return (Vec::new(), cmd_span);
            }
        };
        self.i += 1;

        let mut content = Vec::new();
        let mut depth = 1usize;
        let mut end = open.end;
        while let Some(tok) = self.peek().cloned() {
            match tok.kind {
                TokenKind::LBrace => {
                    depth += 1;
                    self.i += 1;
                }
                TokenKind::RBrace => {
                    depth -= 1;
                    self.i += 1;
                    end = tok.span.end;
                    if depth == 0 {
                        break;
                    }
                }
                TokenKind::Word(w) => {
                    self.i += 1;
                    end = tok.span.end;
                    content.push(Inline::Text {
                        text: w,
                        span: tok.span,
                    });
                }
                TokenKind::Space => {
                    self.i += 1;
                }
                TokenKind::ParBreak => break,
                _ => {
                    self.i += 1;
                }
            }
        }

        if depth != 0 {
            self.diags.push(Diagnostic::error(
                format!("argument to \\{} is missing its closing brace", cmd),
                Some(open),
                Some("closed the argument at end of input".into()),
            ));
        }
        if content.is_empty() {
            self.diags.push(Diagnostic::warning(
                format!("\\{} was given an empty argument", cmd),
                Some(cmd_span.merge(Span::new(open.start, end))),
                Some("nothing was typeset for this command".into()),
            ));
        }
        (content, Span::new(open.start, end))
    }
}
