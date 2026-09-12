//! Parser for the documented LaTeX subset.
//!
//! Honest boundary: this is a finite grammar, not TeX. It recognises the common
//! LaTeX preamble and implements bounded `\newcommand`/`\renewcommand`
//! expansion, but there is no category-code mutation, register, conditional,
//! package loading, or general environment implementation.

use std::collections::{BTreeMap, HashMap};

use crate::diagnostics::Diagnostic;
use crate::lexer::{tokenize, Token, TokenKind};
use crate::math::{self, MathList};
use crate::Span;

/// Maximum number of nested user-macro expansions at one use site.
pub const MACRO_RECURSION_LIMIT: usize = 64;

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

/// A macro definition actually consulted while producing one block.
#[derive(Debug, Clone, PartialEq)]
pub struct MacroDependency {
    pub name: String,
    pub argument_count: usize,
    pub replacement: Vec<TokenKind>,
}

#[derive(Debug)]
pub struct Parsed {
    pub blocks: Vec<Block>,
    pub diagnostics: Vec<Diagnostic>,
    /// The argument of the first valid `\documentclass`, if present.
    pub document_class: Option<String>,
    /// Package names mentioned by valid `\usepackage` commands.
    pub packages: Vec<String>,
    /// One dependency list per block, in `blocks` order.
    pub block_dependencies: Vec<Vec<MacroDependency>>,
    /// Exact preamble bytes. A change invalidates every cached block.
    pub preamble_source: String,
    /// False for recovery/unsupported cases whose state effects are not proven.
    pub incremental_safe: bool,
}

const BUILT_INS: &[&str] = &[
    "section",
    "subsection",
    "textbf",
    "emph",
    "textit",
    "begin",
    "end",
    "par",
    "documentclass",
    "usepackage",
    "newcommand",
    "renewcommand",
];

#[derive(Debug, Clone)]
struct InputToken {
    token: Token,
    expansion_depth: usize,
    maps_to_invocation: bool,
}

#[derive(Debug, Clone)]
struct MacroDef {
    argument_count: usize,
    body: Vec<Token>,
}

pub fn parse(text: &str) -> Parsed {
    let raw = tokenize(text);
    let has_document = has_document_environment(&raw);
    let mut p = P {
        t: raw
            .into_iter()
            .map(|token| InputToken {
                token,
                expansion_depth: 0,
                maps_to_invocation: false,
            })
            .collect(),
        i: 0,
        diags: Vec::new(),
        brace_stack: Vec::new(),
        env_stack: Vec::new(),
        macros: HashMap::new(),
        macro_scopes: Vec::new(),
        has_document,
        in_body: !has_document,
        document_ended: false,
        document_class: None,
        packages: Vec::new(),
        block_dependencies: Vec::new(),
        current_dependencies: BTreeMap::new(),
    };
    let blocks = p.document();

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

    let incremental_safe = p.diags.is_empty();
    Parsed {
        blocks,
        diagnostics: p.diags,
        document_class: p.document_class,
        packages: p.packages,
        block_dependencies: p.block_dependencies,
        preamble_source: preamble_source(text, has_document),
        incremental_safe,
    }
}

struct P {
    t: Vec<InputToken>,
    i: usize,
    diags: Vec<Diagnostic>,
    brace_stack: Vec<Span>,
    env_stack: Vec<(String, Span)>,
    macros: HashMap<String, MacroDef>,
    macro_scopes: Vec<HashMap<String, Option<MacroDef>>>,
    has_document: bool,
    in_body: bool,
    document_ended: bool,
    document_class: Option<String>,
    packages: Vec<String>,
    block_dependencies: Vec<Vec<MacroDependency>>,
    current_dependencies: BTreeMap<String, (usize, Vec<TokenKind>)>,
}

impl P {
    fn peek(&self) -> Option<&Token> {
        self.t.get(self.i).map(|t| &t.token)
    }

    fn document(&mut self) -> Vec<Block> {
        let mut blocks = Vec::new();
        let mut para = Vec::new();

        while self.i < self.t.len() {
            let input = self.t[self.i].clone();
            let tok = input.token;
            let render = self.in_body && !self.document_ended;
            match tok.kind {
                TokenKind::ParBreak => {
                    self.i += 1;
                    if render {
                        self.flush_paragraph(&mut blocks, &mut para);
                    }
                }
                TokenKind::Space | TokenKind::Comment => self.i += 1,
                TokenKind::Word(word) => {
                    self.i += 1;
                    if render {
                        para.push(Inline::Text {
                            text: word,
                            span: tok.span,
                        });
                    }
                }
                TokenKind::LineBreak => {
                    self.i += 1;
                    if render {
                        para.push(Inline::LineBreak { span: tok.span });
                    }
                }
                TokenKind::LBrace => {
                    self.i += 1;
                    self.brace_stack.push(tok.span);
                    self.macro_scopes.push(HashMap::new());
                }
                TokenKind::RBrace => {
                    self.i += 1;
                    if self.brace_stack.pop().is_none() {
                        if render {
                            self.diags.push(Diagnostic::error(
                                "unmatched '}' — no group is open here",
                                Some(tok.span),
                                Some("ignored the stray brace and continued".into()),
                            ));
                        }
                    } else {
                        self.restore_scope();
                    }
                }
                TokenKind::MathShift if render => self.dollar_math(tok.span, &mut para),
                TokenKind::DisplayMathOpen if render => self.bracket_math(tok.span, &mut para),
                TokenKind::DisplayMathClose if render => {
                    self.i += 1;
                    self.diags.push(Diagnostic::error(
                        "stray \\] has no matching \\[",
                        Some(tok.span),
                        Some("ignored the stray display-math delimiter".into()),
                    ));
                }
                TokenKind::Superscript | TokenKind::Subscript if render => {
                    self.i += 1;
                    self.diags.push(Diagnostic::error(
                        "math script marker used outside math mode",
                        Some(tok.span),
                        Some("ignored the script marker and continued".into()),
                    ));
                }
                TokenKind::MathShift
                | TokenKind::DisplayMathOpen
                | TokenKind::DisplayMathClose
                | TokenKind::Superscript
                | TokenKind::Subscript => self.i += 1,
                TokenKind::Command(name) => {
                    self.i += 1;
                    self.command(
                        &name,
                        tok.span,
                        input.expansion_depth,
                        &mut blocks,
                        &mut para,
                    );
                }
            }
        }

        self.flush_paragraph(&mut blocks, &mut para);
        blocks
    }

    fn command(
        &mut self,
        name: &str,
        span: Span,
        depth: usize,
        blocks: &mut Vec<Block>,
        para: &mut Vec<Inline>,
    ) {
        if self.document_ended {
            return;
        }
        if let Some(definition) = self.macros.get(name).cloned() {
            self.record_macro_read(name, &definition);
            // The command token has already been consumed by the main loop.
            // Remove it before inserting its replacement so math-mode slices
            // cannot accidentally retain and typeset the original command too.
            self.i -= 1;
            self.t.remove(self.i);
            self.expand_macro(name, span, depth, definition);
            return;
        }

        match name {
            "documentclass" => self.document_class(span),
            "usepackage" => self.use_package(span),
            "newcommand" | "renewcommand" => self.define_macro(name, span),
            "begin" | "end" => self.environment(name, span, blocks, para),
            _ if self.has_document && !self.in_body => self.unsupported_preamble(name, span),
            "section" | "subsection" => {
                let level = if name == "section" { 1 } else { 2 };
                let (tokens, _) = self.required_group(name, span);
                self.flush_paragraph(blocks, para);
                let content = self.inlines_from_tokens(tokens);
                if content.is_empty() {
                    // A missing/empty heading is already diagnosed where
                    // applicable and has nothing to position. Do not create an
                    // empty block: incremental block spans require real source.
                    self.current_dependencies.clear();
                } else {
                    blocks.push(Block::Heading { level, content });
                    self.finish_block_dependencies();
                }
            }
            "textbf" | "emph" | "textit" => {
                let (tokens, _) = self.required_group(name, span);
                para.extend(self.inlines_from_tokens(tokens));
            }
            "par" => self.flush_paragraph(blocks, para),
            "input" => self.diags.push(Diagnostic::error(
                "\\input and multi-document inclusion are not implemented",
                Some(span),
                Some("skipped the include and typeset its braced path as plain text".into()),
            )),
            "frac" | "sqrt" => self.diags.push(Diagnostic::error(
                format!("\\{} requires math mode", name),
                Some(span),
                Some("skipped the command and typeset its braced arguments as plain text".into()),
            )),
            other => self.unsupported(other, span),
        }
    }

    fn document_class(&mut self, span: Span) {
        let _options = self.optional_bracket_argument();
        let (tokens, _) = self.required_group("documentclass", span);
        let class = token_text(&tokens).trim().to_string();
        if class.is_empty() {
            self.diags.push(Diagnostic::warning(
                "\\documentclass was given an empty argument",
                Some(span),
                Some("no document class was recorded".into()),
            ));
        } else if self.document_class.is_none() {
            self.document_class = Some(class);
        }
    }

    fn use_package(&mut self, span: Span) {
        let _options = self.optional_bracket_argument();
        let (tokens, argument_span) = self.required_group("usepackage", span);
        let packages: Vec<String> = token_text(&tokens)
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_string)
            .collect();
        if packages.is_empty() {
            self.diags.push(Diagnostic::warning(
                "\\usepackage was given an empty package list",
                Some(span.merge(argument_span)),
                Some("no packages were loaded".into()),
            ));
            return;
        }
        self.packages.extend(packages.iter().cloned());
        self.diags.push(Diagnostic::warning(
            format!(
                "packages {} are recognised but not implemented",
                packages.join(", ")
            ),
            Some(span.merge(argument_span)),
            Some("continued without package-specific commands or formatting".into()),
        ));
    }

    fn define_macro(&mut self, kind: &str, span: Span) {
        let (name_tokens, name_span) = self.required_group(kind, span);
        let macro_name = name_tokens
            .iter()
            .filter(|t| !matches!(t.token.kind, TokenKind::Space | TokenKind::Comment))
            .collect::<Vec<_>>();
        let name = match macro_name.as_slice() {
            [InputToken {
                token:
                    Token {
                        kind: TokenKind::Command(name),
                        ..
                    },
                ..
            }] if !name.is_empty() => name.clone(),
            _ => {
                self.diags.push(Diagnostic::error(
                    format!(
                        "\\{} requires a single command name as its first argument",
                        kind
                    ),
                    Some(name_span),
                    Some("ignored the invalid macro definition".into()),
                ));
                let _ = self.optional_bracket_argument();
                let _ = self.required_group(kind, span);
                return;
            }
        };

        let argument_count = match self.optional_bracket_argument() {
            Some((raw, option_span)) => match raw.trim().parse::<usize>() {
                Ok(count) if count <= 9 => count,
                _ => {
                    self.diags.push(Diagnostic::error(
                        format!("\\{} argument count must be an integer from 0 to 9", kind),
                        Some(option_span),
                        Some("ignored the invalid macro definition".into()),
                    ));
                    let _ = self.required_group(kind, span);
                    return;
                }
            },
            None => 0,
        };
        let (body, _) = self.required_group(kind, span);
        let definition = MacroDef {
            argument_count,
            body: body.into_iter().map(|t| t.token).collect(),
        };

        let already_defined = self.macros.contains_key(&name) || BUILT_INS.contains(&name.as_str());
        let valid = if kind == "newcommand" {
            if already_defined {
                self.diags.push(Diagnostic::error(
                    format!("\\newcommand cannot redefine existing command \\{}", name),
                    Some(span.merge(name_span)),
                    Some("kept the existing command definition".into()),
                ));
                false
            } else {
                true
            }
        } else if already_defined {
            true
        } else {
            self.diags.push(Diagnostic::error(
                format!(
                    "\\renewcommand cannot redefine undefined command \\{}",
                    name
                ),
                Some(span.merge(name_span)),
                Some("ignored the invalid redefinition".into()),
            ));
            false
        };
        if valid {
            self.set_macro(name, definition);
        }
    }

    fn expand_macro(&mut self, name: &str, span: Span, depth: usize, definition: MacroDef) {
        let mut arguments = Vec::new();
        for _ in 0..definition.argument_count {
            let (argument, argument_span) = self.required_group(name, span);
            if argument_span != span
                && argument.iter().all(|token| {
                    matches!(
                        token.token.kind,
                        TokenKind::Space | TokenKind::ParBreak | TokenKind::Comment
                    )
                })
            {
                self.diags.push(Diagnostic::error(
                    format!("macro \\{} received an empty required argument", name),
                    Some(argument_span),
                    Some("substituted an empty argument and continued".into()),
                ));
            }
            arguments.push(argument);
        }
        if depth >= MACRO_RECURSION_LIMIT {
            self.diags.push(Diagnostic::error(
                format!(
                    "macro \\{} exceeded the expansion recursion limit of {}",
                    name, MACRO_RECURSION_LIMIT
                ),
                Some(span),
                Some("stopped expanding this macro invocation".into()),
            ));
            return;
        }

        let next_depth = depth + 1;
        let mut expanded = Vec::new();
        for token in definition.body {
            match token.kind {
                TokenKind::Word(word) => {
                    self.expand_macro_word(&word, span, next_depth, &arguments, &mut expanded)
                }
                kind => expanded.push(InputToken {
                    token: Token { kind, span },
                    expansion_depth: next_depth,
                    maps_to_invocation: true,
                }),
            }
        }
        self.t.splice(self.i..self.i, expanded);
    }

    fn expand_macro_word(
        &mut self,
        word: &str,
        invocation_span: Span,
        depth: usize,
        arguments: &[Vec<InputToken>],
        out: &mut Vec<InputToken>,
    ) {
        let bytes = word.as_bytes();
        let mut literal_start = 0;
        let mut index = 0;
        while index + 1 < bytes.len() {
            let digit = bytes[index + 1];
            if bytes[index] == b'#' && (b'1'..=b'9').contains(&digit) {
                if literal_start < index {
                    out.push(mapped_word(
                        &word[literal_start..index],
                        invocation_span,
                        depth,
                    ));
                }
                let argument_index = usize::from(digit - b'1');
                if let Some(argument) = arguments.get(argument_index) {
                    out.extend(argument.iter().cloned().map(|mut token| {
                        token.expansion_depth = depth;
                        token
                    }));
                } else {
                    self.diags.push(Diagnostic::error(
                        format!(
                            "macro replacement references #{} but that argument is not declared",
                            argument_index + 1
                        ),
                        Some(invocation_span),
                        Some("omitted the unavailable argument".into()),
                    ));
                }
                index += 2;
                literal_start = index;
            } else {
                index += 1;
            }
        }
        if literal_start < word.len() {
            out.push(mapped_word(&word[literal_start..], invocation_span, depth));
        }
    }

    fn environment(
        &mut self,
        kind: &str,
        span: Span,
        blocks: &mut Vec<Block>,
        para: &mut Vec<Inline>,
    ) {
        let (tokens, argument_span) = self.required_group(kind, span);
        let environment = token_text(&tokens).trim().to_string();
        if kind == "begin" {
            if environment == "document" && self.has_document {
                self.in_body = true;
            } else if self.in_body {
                self.diags.push(Diagnostic::warning(
                    format!(
                        "environment '{}' is not implemented; its body is typeset as plain text",
                        environment
                    ),
                    Some(span),
                    Some("typeset the body without the environment's formatting".into()),
                ));
            }
            self.env_stack
                .push((environment, span.merge(argument_span)));
            return;
        }

        match self.env_stack.pop() {
            Some((open, _)) if open == environment => {}
            Some((open, _)) => self.diags.push(Diagnostic::error(
                format!(
                    "\\end{{{}}} does not match \\begin{{{}}}",
                    environment, open
                ),
                Some(span),
                Some("closed the innermost open environment".into()),
            )),
            None => self.diags.push(Diagnostic::error(
                format!("\\end{{{}}} with no matching \\begin", environment),
                Some(span),
                Some("ignored the stray \\end".into()),
            )),
        }
        if environment == "document" && self.has_document {
            self.flush_paragraph(blocks, para);
            self.in_body = false;
            self.document_ended = true;
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
            if self.expand_current_macro() {
                continue;
            }
            if self.t[self.i].token.kind == TokenKind::MathShift {
                let closes = !display
                    || self.t.get(self.i + 1).map(|t| &t.token.kind) == Some(&TokenKind::MathShift);
                if closes {
                    content_end = self.i;
                    close_end = if display {
                        self.t[self.i + 1].token.span.end
                    } else {
                        self.t[self.i].token.span.end
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
        while self.i < self.t.len() {
            if self.expand_current_macro() {
                continue;
            }
            if self.t[self.i].token.kind == TokenKind::DisplayMathClose {
                break;
            }
            self.i += 1;
        }
        let content_end = self.i;
        let found = self.i < self.t.len();
        let close_end = if found {
            let end = self.t[self.i].token.span.end;
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

    fn expand_current_macro(&mut self) -> bool {
        let Some(input) = self.t.get(self.i).cloned() else {
            return false;
        };
        let TokenKind::Command(name) = &input.token.kind else {
            return false;
        };
        let Some(definition) = self.macros.get(name).cloned() else {
            return false;
        };
        self.record_macro_read(name, &definition);
        self.t.remove(self.i);
        self.expand_macro(name, input.token.span, input.expansion_depth, definition);
        true
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
        // Unterminated math inside an expansion can report a content end past the
        // token stream: the closing token the caller expected was never produced.
        // Clamp rather than slice out of range — the diagnostic for the unclosed
        // construct is emitted by the caller either way.
        let content_start = content_start.min(self.t.len());
        let content_end = content_end.clamp(content_start, self.t.len());
        let mut raw = Vec::new();
        for input in &self.t[content_start..content_end] {
            if input.maps_to_invocation {
                if let TokenKind::Word(word) = &input.token.kind {
                    for ch in word.chars() {
                        raw.push(Token {
                            kind: TokenKind::Word(ch.to_string()),
                            span: input.token.span,
                        });
                    }
                    continue;
                }
            }
            raw.push(input.token.clone());
        }
        let list = math::parse_tokens(&raw, &mut self.diags);
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

    fn required_group(&mut self, command: &str, command_span: Span) -> (Vec<InputToken>, Span) {
        self.skip_spaces();
        let open = match self.peek() {
            Some(token) if token.kind == TokenKind::LBrace => token.span,
            _ => {
                self.diags.push(Diagnostic::error(
                    format!("\\{} requires a braced argument", command),
                    Some(command_span),
                    Some("used an empty argument and continued".into()),
                ));
                return (Vec::new(), command_span);
            }
        };
        self.i += 1;
        let start = self.i;
        let mut depth = 1usize;
        let mut end = open.end;
        while self.i < self.t.len() {
            let token = &self.t[self.i].token;
            match token.kind {
                TokenKind::LBrace => depth += 1,
                TokenKind::RBrace => {
                    depth -= 1;
                    if depth == 0 {
                        end = token.span.end;
                        let content = self.t[start..self.i].to_vec();
                        self.i += 1;
                        return (content, Span::new(open.start, end));
                    }
                }
                _ => {}
            }
            end = token.span.end;
            self.i += 1;
        }
        self.diags.push(Diagnostic::error(
            format!("argument to \\{} is missing its closing brace", command),
            Some(open),
            Some("closed the argument at end of input".into()),
        ));
        (self.t[start..].to_vec(), Span::new(open.start, end))
    }

    /// Brackets stay ordinary lexer word characters, preserving normal text.
    fn optional_bracket_argument(&mut self) -> Option<(String, Span)> {
        self.skip_spaces();
        let first = self.peek()?;
        let TokenKind::Word(first_word) = &first.kind else {
            return None;
        };
        if !first_word.starts_with('[') {
            return None;
        }
        let start = first.span.start;
        let mut end = first.span.end;
        let mut found = first_word.contains(']');
        let mut raw = first_word.clone();
        self.i += 1;
        while !found && self.i < self.t.len() {
            let token = &self.t[self.i].token;
            end = token.span.end;
            match &token.kind {
                TokenKind::Word(word) => {
                    raw.push_str(word);
                    found = word.contains(']');
                }
                TokenKind::Space | TokenKind::ParBreak => raw.push(' '),
                TokenKind::Command(name) => {
                    raw.push('\\');
                    raw.push_str(name);
                }
                _ => {}
            }
            self.i += 1;
        }
        let span = Span::new(start, end);
        let content = raw
            .strip_prefix('[')
            .unwrap_or(&raw)
            .split_once(']')
            .map_or(raw.as_str(), |(inside, _)| inside)
            .to_string();
        if !found {
            self.diags.push(Diagnostic::error(
                "optional argument is missing its closing ']'",
                Some(span),
                Some("used the text through end of input as the option".into()),
            ));
        }
        Some((content, span))
    }

    fn inlines_from_tokens(&mut self, tokens: Vec<InputToken>) -> Vec<Inline> {
        let outer_tokens = std::mem::replace(&mut self.t, tokens);
        let outer_index = std::mem::replace(&mut self.i, 0);
        let mut expanded = Vec::new();
        while self.i < self.t.len() {
            if self.expand_current_macro() {
                continue;
            }
            expanded.push(self.t[self.i].clone());
            self.i += 1;
        }
        self.t = outer_tokens;
        self.i = outer_index;

        let mut content = Vec::new();
        for input in expanded {
            match input.token.kind {
                TokenKind::Word(text) => content.push(Inline::Text {
                    text,
                    span: input.token.span,
                }),
                TokenKind::LineBreak => content.push(Inline::LineBreak {
                    span: input.token.span,
                }),
                _ => {}
            }
        }
        content
    }

    fn set_macro(&mut self, name: String, definition: MacroDef) {
        if let Some(scope) = self.macro_scopes.last_mut() {
            scope
                .entry(name.clone())
                .or_insert_with(|| self.macros.get(&name).cloned());
        }
        self.macros.insert(name, definition);
    }

    fn restore_scope(&mut self) {
        if let Some(scope) = self.macro_scopes.pop() {
            for (name, previous) in scope {
                match previous {
                    Some(definition) => {
                        self.macros.insert(name, definition);
                    }
                    None => {
                        self.macros.remove(&name);
                    }
                }
            }
        }
    }

    fn record_macro_read(&mut self, name: &str, definition: &MacroDef) {
        self.current_dependencies.insert(
            name.to_string(),
            (
                definition.argument_count,
                definition
                    .body
                    .iter()
                    .map(|token| token.kind.clone())
                    .collect(),
            ),
        );
    }

    fn finish_block_dependencies(&mut self) {
        self.block_dependencies.push(
            std::mem::take(&mut self.current_dependencies)
                .into_iter()
                .map(|(name, (argument_count, replacement))| MacroDependency {
                    name,
                    argument_count,
                    replacement,
                })
                .collect(),
        );
    }

    fn flush_paragraph(&mut self, blocks: &mut Vec<Block>, paragraph: &mut Vec<Inline>) {
        if !paragraph.is_empty() {
            blocks.push(Block::Paragraph(std::mem::take(paragraph)));
            self.finish_block_dependencies();
        }
    }

    fn skip_spaces(&mut self) {
        while matches!(
            self.peek().map(|token| &token.kind),
            Some(TokenKind::Space | TokenKind::Comment)
        ) {
            self.i += 1;
        }
    }

    fn unsupported_preamble(&mut self, name: &str, span: Span) {
        self.diags.push(Diagnostic::error(
            format!("\\{} is not supported in the document preamble", name),
            Some(span),
            Some("skipped the command and did not typeset preamble content".into()),
        ));
    }

    fn unsupported(&mut self, name: &str, span: Span) {
        debug_assert!(!BUILT_INS.contains(&name));
        self.diags.push(Diagnostic::error(
            format!(
                "\\{} is not supported by this compiler version; unrestricted TeX math mode is not implemented",
                name
            ),
            Some(span),
            Some("skipped the command; any braced argument was typeset as plain text".into()),
        ));
    }
}

fn mapped_word(word: &str, span: Span, depth: usize) -> InputToken {
    InputToken {
        token: Token {
            kind: TokenKind::Word(word.to_string()),
            span,
        },
        expansion_depth: depth,
        maps_to_invocation: true,
    }
}

fn preamble_source(text: &str, has_document: bool) -> String {
    let tokens = tokenize(text);
    let end = if has_document {
        tokens.iter().enumerate().find_map(|(index, token)| {
            if token.kind != TokenKind::Command("begin".into()) {
                return None;
            }
            let significant: Vec<&Token> = tokens[index + 1..]
                .iter()
                .filter(|token| !matches!(token.kind, TokenKind::Space | TokenKind::Comment))
                .take(3)
                .collect();
            match significant.as_slice() {
                [Token {
                    kind: TokenKind::LBrace,
                    ..
                }, Token {
                    kind: TokenKind::Word(name),
                    ..
                }, Token {
                    kind: TokenKind::RBrace,
                    span,
                }] if name == "document" => Some(span.end),
                _ => None,
            }
        })
    } else if tokens.iter().any(|token| {
        matches!(
            &token.kind,
            TokenKind::Command(name) if name == "documentclass" || name == "usepackage"
        )
    }) {
        Some(text.len())
    } else {
        None
    };
    end.map_or("", |end| &text[..end]).to_string()
}

fn token_text(tokens: &[InputToken]) -> String {
    let mut result = String::new();
    for input in tokens {
        match &input.token.kind {
            TokenKind::Word(text) | TokenKind::Command(text) => result.push_str(text),
            TokenKind::Space | TokenKind::ParBreak => result.push(' '),
            _ => {}
        }
    }
    result
}

fn has_document_environment(tokens: &[Token]) -> bool {
    tokens.iter().enumerate().any(|(index, token)| {
        if token.kind != TokenKind::Command("begin".into()) {
            return false;
        }
        let significant: Vec<&Token> = tokens[index + 1..]
            .iter()
            .filter(|token| !matches!(token.kind, TokenKind::Space | TokenKind::Comment))
            .take(3)
            .collect();
        matches!(
            significant.as_slice(),
            [
                Token { kind: TokenKind::LBrace, .. },
                Token { kind: TokenKind::Word(name), .. },
                Token { kind: TokenKind::RBrace, .. }
            ] if name == "document"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout;

    fn items(source: &str) -> (Parsed, Vec<crate::layout::TextItem>) {
        let parsed = parse(source);
        let items = layout::layout(&parsed.blocks)
            .into_iter()
            .flat_map(|page| page.items)
            .collect();
        (parsed, items)
    }

    #[test]
    fn preamble_is_recorded_and_only_document_body_is_typeset() {
        let source = "\\documentclass[draft]{article}\n\\usepackage[demo]{amsmath}\n\\begin{document}Body only\\end{document}trailer";
        let (parsed, items) = items(source);
        assert_eq!(parsed.document_class.as_deref(), Some("article"));
        assert_eq!(parsed.packages, ["amsmath"]);
        assert_eq!(
            items
                .iter()
                .map(|item| item.text.as_str())
                .collect::<Vec<_>>(),
            ["Body", "only"]
        );
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(parsed.diagnostics[0].message.contains("amsmath"));
    }

    #[test]
    fn zero_argument_macro_maps_literal_output_to_invocation() {
        let source = "\\newcommand{\\hi}{Hello} \\hi";
        let (parsed, items) = items(source);
        assert!(parsed.diagnostics.is_empty());
        let hello = items.iter().find(|item| item.text == "Hello").unwrap();
        let start = source.rfind("\\hi").unwrap();
        assert_eq!(hello.span, Span::new(start, start + "\\hi".len()));
    }

    #[test]
    fn macro_argument_keeps_argument_source_span() {
        let source = "\\newcommand{\\greet}[1]{Hello #1} \\greet{world}";
        let (parsed, items) = items(source);
        assert!(parsed.diagnostics.is_empty());
        assert!(items.iter().any(|item| item.text == "Hello"));
        let world = items.iter().find(|item| item.text == "world").unwrap();
        let start = source.rfind("world").unwrap();
        assert_eq!(world.span, Span::new(start, start + "world".len()));
    }

    #[test]
    fn nested_macros_expand_and_renewcommand_replaces_an_existing_macro() {
        let source = r"\newcommand{\inner}{first} \newcommand{\outer}{\inner} \outer \renewcommand{\inner}{second} \outer";
        let (parsed, items) = items(source);
        assert!(parsed.diagnostics.is_empty());
        assert_eq!(
            items
                .iter()
                .map(|item| item.text.as_str())
                .collect::<Vec<_>>(),
            ["first", "second"]
        );
    }

    #[test]
    fn invalid_newcommand_and_renewcommand_relationships_are_diagnostic() {
        let source =
            r"\newcommand{\same}{old}\newcommand{\same}{new}\renewcommand{\missing}{body}\same";
        let (parsed, items) = items(source);
        assert_eq!(
            items
                .iter()
                .map(|item| item.text.as_str())
                .collect::<Vec<_>>(),
            ["old"]
        );
        assert!(parsed.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains(r"\newcommand cannot redefine existing command \same")));
        assert!(parsed.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains(r"\renewcommand cannot redefine undefined command \missing")));
    }

    #[test]
    fn macros_expand_inside_supported_command_arguments() {
        let source = r"\newcommand{\titleword}{Title}\section{\titleword}";
        let (parsed, items) = items(source);
        assert!(parsed.diagnostics.is_empty());
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].text, "Title");
        let start = source.rfind(r"\titleword").unwrap();
        assert_eq!(items[0].span, Span::new(start, start + r"\titleword".len()));
    }

    #[test]
    fn macro_expansion_in_math_does_not_fabricate_per_glyph_spans() {
        let source = r"\newcommand{\pair}{abcde} $\pair$";
        let (parsed, items) = items(source);
        assert!(parsed.diagnostics.is_empty());
        let invocation_start = source.rfind(r"\pair").unwrap();
        let invocation_span = Span::new(invocation_start, invocation_start + r"\pair".len());
        let math_items: Vec<_> = items
            .iter()
            .filter(|item| "abcde".contains(item.text.as_str()))
            .collect();
        assert_eq!(math_items.len(), 5);
        assert!(
            math_items.iter().all(|item| item.span == invocation_span),
            "expected {invocation_span:?}, got {:?}",
            math_items.iter().map(|item| item.span).collect::<Vec<_>>()
        );
    }

    #[test]
    fn group_local_macro_is_restored_when_the_group_closes() {
        let source = "{\\newcommand{\\local}{inside} \\local} \\local";
        let (parsed, items) = items(source);
        assert_eq!(items.iter().filter(|item| item.text == "inside").count(), 1);
        assert!(parsed
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("\\local is not supported")));
    }

    #[test]
    fn self_referential_macro_hits_explicit_recursion_limit() {
        let source = "\\newcommand{\\loop}{\\loop} \\loop";
        let (parsed, _) = items(source);
        assert!(parsed.diagnostics.iter().any(|diagnostic| {
            diagnostic.message.contains("\\loop")
                && diagnostic
                    .message
                    .contains(&MACRO_RECURSION_LIMIT.to_string())
        }));
    }
}
