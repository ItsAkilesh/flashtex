//! Parser for the documented LaTeX subset.
//!
//! Honest boundary: this is a finite grammar, not TeX. It recognises the common
//! LaTeX preamble and implements bounded `\newcommand`/`\renewcommand`
//! expansion, but there is no category-code mutation, register, conditional,
//! package loading, or general environment implementation.

use std::collections::{BTreeMap, HashMap};

use crate::diagnostics::Diagnostic;
use crate::lexer::{tokenize, tokenize_document, Token, TokenKind};
use crate::math::{self, MathList};
use crate::{DocumentId, Span};

/// Maximum number of nested user-macro expansions at one use site.
pub const MACRO_RECURSION_LIMIT: usize = 64;
/// Maximum number of active nested `\input`/`\include` calls.
pub const INCLUDE_DEPTH_LIMIT: usize = 64;

/// One project document supplied by the runtime compile payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceDocument<'a> {
    pub path: &'a str,
    pub text: &'a str,
}

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
        number: Option<String>,
        number_span: Option<Span>,
        span: Span,
    },
    /// A multi-row amsmath display (`gather`, `align` and their starred forms).
    /// `aligned` cells alternate right/left alignment around shared tab stops.
    MathRows {
        rows: Vec<MathRow>,
        aligned: bool,
        span: Span,
    },
    Label {
        key: String,
        value: String,
        span: Span,
    },
    Reference {
        key: String,
        page: bool,
        span: Span,
    },
}

/// One `\\`-separated row of a multi-row display; cells are split on `&`.
#[derive(Debug, Clone, PartialEq)]
pub struct MathRow {
    pub cells: Vec<MathList>,
    pub number: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Paragraph(Vec<Inline>),
    Heading {
        level: u8,
        number: String,
        number_span: Span,
        content: Vec<Inline>,
    },
    FigureCaption {
        content: Vec<Inline>,
    },
    /// A paragraph inside `center`, `flushleft`, `flushright`, `quote` or
    /// `quotation`.
    Styled {
        style: ParagraphStyle,
        content: Vec<Inline>,
    },
    /// `\vspace{<dimen>}`: additional vertical glue, in points.
    VSpace {
        pt: f64,
    },
    /// `\hrule`: a full-measure-width rule at the current line.
    Rule {
        span: Span,
    },
    /// `\newpage`: force the next block onto a fresh page.
    PageBreak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParagraphStyle {
    Center,
    FlushRight,
    FlushLeft,
    /// `quote`/`quotation`: both margins indented.
    Quote,
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
    /// True when counters or the label table make layout document-global.
    pub document_global_state: bool,
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
    "setlist",
    "newcommand",
    "renewcommand",
    "input",
    "include",
    "label",
    "ref",
    "pageref",
    "caption",
    "item",
    "includegraphics",
    "hfill",
    "normalfont",
    "bfseries",
    "listfiles",
    "noindent",
    "vspace",
    "hrule",
    "newpage",
    "pagestyle",
];

/// Parses a LaTeX dimension (`12pt`, `1.5em`, `0.5in`, `2cm`, `10mm`) to points.
/// `em` is relative to the compiler's fixed body size since there is no
/// declaration-scoped font state to read a current size from (see the
/// `hfill`/`normalfont`/`bfseries` comment below).
pub(crate) fn parse_dimen_pt(text: &str) -> Option<f64> {
    let text = text.trim();
    let unit_len = text
        .chars()
        .rev()
        .take_while(|c| c.is_ascii_alphabetic())
        .count();
    if unit_len == 0 || unit_len > text.len() {
        return None;
    }
    let split = text.len() - unit_len;
    let (number, unit) = text.split_at(split);
    let value: f64 = number.trim().parse().ok()?;
    let per_pt = match unit {
        "pt" => 1.0,
        "in" => 72.27,
        "cm" => 72.27 / 2.54,
        "mm" => 72.27 / 25.4,
        "em" => crate::layout::BODY_SIZE_PT,
        _ => return None,
    };
    Some(value * per_pt)
}

/// Project-relative paths only: no absolute paths or parent traversal.
pub(crate) fn path_is_safe(path: &str) -> bool {
    if path.is_empty() || path.starts_with('/') || path.starts_with('\\') {
        return false;
    }
    if path.len() >= 2 && path.as_bytes()[1] == b':' {
        return false;
    }
    !path.split(['/', '\\']).any(|component| component == "..")
}

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
    parse_project(&[SourceDocument { path: "", text }], "")
}

/// Parse an entry document and every project document it includes.
pub fn parse_project(documents: &[SourceDocument<'_>], entry_path: &str) -> Parsed {
    let entry = documents
        .iter()
        .position(|document| document.path == entry_path)
        .unwrap_or(0);
    let entry_document = documents.get(entry).copied().unwrap_or(SourceDocument {
        path: entry_path,
        text: "",
    });
    let raw = tokenize_document(entry_document.text, DocumentId(entry));
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
        documents,
        document_by_path: documents
            .iter()
            .enumerate()
            .map(|(index, document)| (document.path, index))
            .collect(),
        include_stack: vec![entry],
        section_counter: 0,
        subsection_counter: 0,
        equation_counter: 0,
        figure_counter: 0,
        current_counter: None,
        seen_labels: HashMap::new(),
        list_stack: Vec::new(),
        paragraph_styles: Vec::new(),
        document_global_state: false,
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
        preamble_source: preamble_source(entry_document.text, has_document),
        incremental_safe,
        document_global_state: p.document_global_state,
    }
}

struct P<'a> {
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
    documents: &'a [SourceDocument<'a>],
    document_by_path: HashMap<&'a str, usize>,
    include_stack: Vec<usize>,
    section_counter: u32,
    subsection_counter: u32,
    equation_counter: u32,
    figure_counter: u32,
    current_counter: Option<String>,
    seen_labels: HashMap<String, Span>,
    /// Environment name, item count, and an enumitem label template if given.
    list_stack: Vec<(String, u32, Option<String>)>,
    paragraph_styles: Vec<ParagraphStyle>,
    document_global_state: bool,
}

impl P<'_> {
    fn peek(&self) -> Option<&Token> {
        self.t.get(self.i).map(|t| &t.token)
    }

    fn document(&mut self) -> Vec<Block> {
        let mut blocks = Vec::new();
        let mut para = Vec::new();

        self.parse_stream(&mut blocks, &mut para);
        self.flush_paragraph(&mut blocks, &mut para);
        blocks
    }

    fn parse_stream(&mut self, blocks: &mut Vec<Block>, para: &mut Vec<Inline>) {
        while self.i < self.t.len() {
            let input = self.t[self.i].clone();
            let tok = input.token;
            let render = self.in_body && !self.document_ended;
            match tok.kind {
                TokenKind::ParBreak => {
                    self.i += 1;
                    if render {
                        self.flush_paragraph(blocks, para);
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
                    // `\\[<length>]`: the vertical space is not modelled, but the
                    // argument must not be typeset as text.
                    self.skip_line_break_length();
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
                TokenKind::MathShift if render => self.dollar_math(tok.span, para),
                TokenKind::DisplayMathOpen if render => self.bracket_math(tok.span, para),
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
                    self.command(&name, tok.span, input.expansion_depth, blocks, para);
                }
            }
        }
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
            // The command token has already been consumed by the main loop. Keep
            // its index while arguments are consumed, then replace the complete
            // invocation with one splice.
            let invocation_start = self.i - 1;
            self.expand_macro(name, span, depth, definition, invocation_start);
            return;
        }

        match name {
            "documentclass" => self.document_class(span),
            "usepackage" => self.use_package(span),
            "setlist" => self.set_list(span),
            "newcommand" | "renewcommand" => self.define_macro(name, span),
            "begin" | "end" => self.environment(name, span, blocks, para),
            "input" | "include" => self.include(name, span, blocks, para),
            // MacTeX writes package-version banners to the log for `\listfiles`;
            // this compiler has no log stream to write them to, so the honest
            // behaviour is a documented no-op rather than an "unsupported"
            // diagnostic for a command every corpus fixture's preamble carries.
            "listfiles" => {}
            _ if self.has_document && !self.in_body => self.unsupported_preamble(name, span),
            "section" | "subsection" => {
                let level = if name == "section" { 1 } else { 2 };
                let starred = self.take_optional_star();
                let (tokens, _) = self.required_group(name, span);
                self.flush_paragraph(blocks, para);
                let number = if starred {
                    String::new()
                } else if level == 1 {
                    self.section_counter += 1;
                    self.subsection_counter = 0;
                    self.section_counter.to_string()
                } else {
                    self.subsection_counter += 1;
                    format!("{}.{}", self.section_counter, self.subsection_counter)
                };
                if !starred {
                    self.current_counter = Some(number.clone());
                }
                let content = self.inlines_from_tokens(tokens);
                if content.is_empty() {
                    // A missing/empty heading is already diagnosed where
                    // applicable and has nothing to position. Do not create an
                    // empty block: incremental block spans require real source.
                    self.current_dependencies.clear();
                } else {
                    blocks.push(Block::Heading {
                        level,
                        number,
                        number_span: span,
                        content,
                    });
                    self.finish_block_dependencies();
                }
            }
            "label" => {
                let (tokens, argument_span) = self.required_group(name, span);
                let key = token_text(&tokens).trim().to_string();
                self.document_global_state = true;
                if key.is_empty() {
                    self.diags.push(Diagnostic::warning(
                        "\\label was given an empty key",
                        Some(span.merge(argument_span)),
                        Some("ignored the empty label".into()),
                    ));
                } else {
                    if self.seen_labels.insert(key.clone(), span).is_some() {
                        self.diags.push(Diagnostic::warning(
                            format!("duplicate \\label{{{key}}}; the second definition wins"),
                            Some(span.merge(argument_span)),
                            Some("replaced the earlier label definition".into()),
                        ));
                    }
                    para.push(Inline::Label {
                        key,
                        value: self.current_counter.clone().unwrap_or_default(),
                        span,
                    });
                }
            }
            "ref" | "pageref" => {
                let (tokens, argument_span) = self.required_group(name, span);
                let key = token_text(&tokens).trim().to_string();
                self.document_global_state = true;
                para.push(Inline::Reference {
                    key,
                    page: name == "pageref",
                    span: span.merge(argument_span),
                });
            }
            "caption" => {
                let (tokens, _) = self.required_group(name, span);
                if self.env_stack.last().map(|(name, _)| name.as_str()) != Some("figure") {
                    self.diags.push(Diagnostic::error(
                        "\\caption is only supported inside a figure environment",
                        Some(span),
                        Some("typeset the caption text as an ordinary paragraph".into()),
                    ));
                    para.extend(self.inlines_from_tokens(tokens));
                } else {
                    self.flush_paragraph(blocks, para);
                    self.figure_counter += 1;
                    self.current_counter = Some(self.figure_counter.to_string());
                    let mut content = vec![Inline::Text {
                        text: format!("Figure {}:", self.figure_counter),
                        span,
                    }];
                    content.extend(self.inlines_from_tokens(tokens));
                    blocks.push(Block::FigureCaption { content });
                    self.finish_block_dependencies();
                }
            }
            "item" => {
                self.flush_paragraph(blocks, para);
                match self.list_stack.last_mut() {
                    Some((kind, count, template)) => {
                        *count += 1;
                        let marker = if kind == "enumerate" {
                            match template {
                                Some(template) => enumitem_label(template, *count),
                                None => format!("{}.", count),
                            }
                        } else {
                            "•".to_string()
                        };
                        para.push(Inline::Text { text: marker, span });
                    }
                    None => self.diags.push(Diagnostic::error(
                        "\\item is only supported inside itemize or enumerate",
                        Some(span),
                        Some("ignored the item marker and continued".into()),
                    )),
                }
            }
            "includegraphics" => {
                let _ = self.optional_bracket_argument();
                let _ = self.required_group(name, span);
                self.diags.push(Diagnostic::warning(
                    "\\includegraphics is unsupported; image loading is not implemented",
                    Some(span),
                    Some("omitted the image and continued".into()),
                ));
            }
            "textbf" | "emph" | "textit" => {
                let (tokens, _) = self.required_group(name, span);
                para.extend(self.inlines_from_tokens(tokens));
            }
            // The current layout model has no stretchable horizontal glue or
            // declaration-scoped font state. These commands are explicit no-ops:
            // they never consume or alter surrounding content.
            "hfill" | "normalfont" | "bfseries" => {}
            // No paragraph is ever given a first-line indent in this layout
            // model, so there is nothing for \noindent to suppress: an honest
            // no-op rather than a fabricated indent to cancel.
            "noindent" => {}
            "par" => self.flush_paragraph(blocks, para),
            "vspace" => {
                let (tokens, argument_span) = self.required_group(name, span);
                let raw = token_text(&tokens);
                match parse_dimen_pt(&raw) {
                    Some(pt) => {
                        self.flush_paragraph(blocks, para);
                        blocks.push(Block::VSpace { pt });
                        self.finish_block_dependencies();
                    }
                    None => self.diags.push(Diagnostic::error(
                        format!(
                            "\\vspace requires a recognised dimension, got '{}'",
                            raw.trim()
                        ),
                        Some(span.merge(argument_span)),
                        Some("ignored the vertical space and continued".into()),
                    )),
                }
            }
            "hrule" => {
                self.flush_paragraph(blocks, para);
                blocks.push(Block::Rule { span });
                self.finish_block_dependencies();
            }
            "newpage" => {
                self.flush_paragraph(blocks, para);
                blocks.push(Block::PageBreak);
                self.finish_block_dependencies();
            }
            "pagestyle" => {
                // No header/footer rendering exists yet, so every style is
                // accepted with the same (honest) effect: none. `empty` and
                // `plain` both describe "no footer content beyond a page
                // number", which is already what happens.
                let _ = self.required_group(name, span);
            }
            "frac" | "sqrt" => self.diags.push(Diagnostic::error(
                format!("\\{} requires math mode", name),
                Some(span),
                Some("skipped the command and typeset its braced arguments as plain text".into()),
            )),
            other => self.unsupported(other, span),
        }
    }

    fn include(
        &mut self,
        command: &str,
        span: Span,
        blocks: &mut Vec<Block>,
        para: &mut Vec<Inline>,
    ) {
        let (tokens, _) = self.required_group(command, span);
        let requested = token_text(&tokens).trim().to_string();
        if requested.is_empty() {
            self.diags.push(Diagnostic::error(
                format!("\\{command} requires a non-empty project-relative path"),
                Some(span),
                Some("skipped the empty include and continued".into()),
            ));
            return;
        }
        if !path_is_safe(&requested) {
            self.diags.push(Diagnostic::error(
                format!(
                    "rejected include path '{requested}': paths must be project-relative with no parent traversal"
                ),
                Some(span),
                Some("skipped the unsafe include and continued".into()),
            ));
            return;
        }

        let appended = format!("{requested}.tex");
        let resolved = self
            .document_by_path
            .get(requested.as_str())
            .copied()
            .or_else(|| self.document_by_path.get(appended.as_str()).copied());
        let Some(document_index) = resolved else {
            self.diags.push(Diagnostic::error(
                format!("included file not found: looked for '{requested}' and '{appended}'"),
                Some(span),
                Some("skipped the missing include and continued".into()),
            ));
            return;
        };

        if let Some(cycle_start) = self
            .include_stack
            .iter()
            .position(|active| *active == document_index)
        {
            let mut cycle: Vec<&str> = self.include_stack[cycle_start..]
                .iter()
                .map(|index| self.documents[*index].path)
                .collect();
            cycle.push(self.documents[document_index].path);
            self.diags.push(Diagnostic::error(
                format!("include cycle detected: {}", cycle.join(" -> ")),
                Some(span),
                Some("skipped the cyclic include and continued".into()),
            ));
            return;
        }
        if self.include_stack.len() > INCLUDE_DEPTH_LIMIT {
            self.diags.push(Diagnostic::error(
                format!(
                    "include depth exceeds the limit of {INCLUDE_DEPTH_LIMIT} while loading '{}'",
                    self.documents[document_index].path
                ),
                Some(span),
                Some("skipped the too-deep include and continued".into()),
            ));
            return;
        }

        let saved_tokens = std::mem::replace(
            &mut self.t,
            tokenize_document(
                self.documents[document_index].text,
                DocumentId(document_index),
            )
            .into_iter()
            .map(|token| InputToken {
                token,
                expansion_depth: 0,
                maps_to_invocation: false,
            })
            .collect(),
        );
        let saved_index = std::mem::replace(&mut self.i, 0);
        self.include_stack.push(document_index);
        self.parse_stream(blocks, para);
        self.include_stack.pop();
        self.t = saved_tokens;
        self.i = saved_index;
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

    fn set_list(&mut self, span: Span) {
        let _ = self.optional_bracket_argument();
        let (_, argument_span) = self.required_group("setlist", span);
        self.diags.push(Diagnostic::warning(
            "\\setlist list spacing is recognised but not implemented",
            Some(span.merge(argument_span)),
            Some("lists use the compiler's default spacing".into()),
        ));
    }

    fn use_package(&mut self, span: Span) {
        let options = self
            .optional_bracket_argument()
            .map(|(options, _)| options)
            .unwrap_or_default();
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
        let packages: Vec<String> = packages
            .into_iter()
            .filter(|package| !package_matches_layout(package, &options))
            .collect();
        if packages.is_empty() {
            return;
        }
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

    fn expand_macro(
        &mut self,
        name: &str,
        span: Span,
        depth: usize,
        definition: MacroDef,
        invocation_start: usize,
    ) {
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
            self.t.drain(invocation_start..self.i);
            self.i = invocation_start;
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
        // One shift, not two. Callers used to `remove` the invocation token and
        // then `splice` the expansion into the gap, so every macro invocation
        // moved the tail of the token vector twice. Callers now leave the
        // invocation in place and this replaces it in a single splice.
        //
        // This halves the work but the operation is still linear in the tokens
        // after the cursor, so parsing remains superlinear in macro-dense
        // documents. Measured: parse is 402 ms of a 420 ms edit at 500 KB.
        // The real fix is incremental parsing, which is a larger change than
        // this revision's scope; the README records the measurement.
        let invocation_end = self.i;
        self.t.splice(invocation_start..invocation_end, expanded);
        self.i = invocation_start;
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
            if matches!(
                environment.as_str(),
                "equation" | "equation*" | "displaymath"
            ) && self.in_body
            {
                self.equation_environment(span, &environment, blocks, para);
                return;
            }
            if matches!(
                environment.as_str(),
                "gather" | "gather*" | "align" | "align*"
            ) && self.in_body
            {
                self.multirow_environment(span, &environment, blocks, para);
                return;
            }
            if environment == "document" && self.has_document {
                self.in_body = true;
            } else if environment == "figure" && self.in_body {
                self.flush_paragraph(blocks, para);
            } else if let (Some(style), true) = (paragraph_style(&environment), self.in_body) {
                self.flush_paragraph(blocks, para);
                self.paragraph_styles.push(style);
            } else if matches!(environment.as_str(), "itemize" | "enumerate") && self.in_body {
                self.flush_paragraph(blocks, para);
                let template = self.optional_bracket_argument().map(|(options, _)| options);
                self.list_stack.push((environment.clone(), 0, template));
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
        if paragraph_style(&environment).is_some() && self.in_body {
            self.flush_paragraph(blocks, para);
            self.paragraph_styles.pop();
        } else if matches!(environment.as_str(), "itemize" | "enumerate") {
            self.flush_paragraph(blocks, para);
            self.list_stack.pop();
        } else if environment == "figure" {
            self.flush_paragraph(blocks, para);
        }
        if environment == "document" && self.has_document {
            self.flush_paragraph(blocks, para);
            self.in_body = false;
            self.document_ended = true;
        }
    }

    fn equation_environment(
        &mut self,
        open: Span,
        name: &str,
        blocks: &mut Vec<Block>,
        para: &mut Vec<Inline>,
    ) {
        self.flush_paragraph(blocks, para);
        let numbered = name == "equation";
        if numbered {
            self.equation_counter += 1;
            self.current_counter = Some(self.equation_counter.to_string());
        }
        let number = self.equation_counter.to_string();
        let mut raw = Vec::new();
        let mut labels = Vec::new();
        let mut end = open.end;
        let mut found_end = false;

        while self.i < self.t.len() {
            if self.expand_current_macro() {
                continue;
            }
            if let Some((after, end_span)) = environment_end_at(&self.t, self.i, name) {
                self.i = after;
                end = end_span.end;
                found_end = true;
                break;
            }
            if matches!(&self.t[self.i].token.kind, TokenKind::Command(name) if name == "label") {
                let label_span = self.t[self.i].token.span;
                self.i += 1;
                let (tokens, argument_span) = self.required_group("label", label_span);
                let key = token_text(&tokens).trim().to_string();
                self.document_global_state = true;
                if !key.is_empty() {
                    if self.seen_labels.insert(key.clone(), label_span).is_some() {
                        self.diags.push(Diagnostic::warning(
                            format!("duplicate \\label{{{key}}}; the second definition wins"),
                            Some(label_span.merge(argument_span)),
                            Some("replaced the earlier label definition".into()),
                        ));
                    }
                    labels.push(Inline::Label {
                        key,
                        value: number.clone(),
                        span: label_span,
                    });
                }
                continue;
            }
            end = self.t[self.i].token.span.end;
            raw.push(self.t[self.i].token.clone());
            self.i += 1;
        }
        if !found_end {
            self.diags.push(Diagnostic::error(
                format!("unterminated environment '{name}' — no matching \\end"),
                Some(open),
                Some("closed the equation at end of input".into()),
            ));
        }
        let list = math::parse_tokens(&raw, &mut self.diags);
        para.push(Inline::Math {
            list,
            display: true,
            number: numbered.then_some(number),
            number_span: numbered.then_some(open),
            span: Span::in_document(open.document, open.start, end),
        });
        para.extend(labels);
        self.flush_paragraph(blocks, para);
    }

    /// amsmath `gather`/`align` (and starred forms): rows split on top-level
    /// `\\`, `align` cells split on top-level `&`. Numbered forms number every
    /// row except those carrying `\nonumber`/`\notag`.
    fn multirow_environment(
        &mut self,
        open: Span,
        name: &str,
        blocks: &mut Vec<Block>,
        para: &mut Vec<Inline>,
    ) {
        self.flush_paragraph(blocks, para);
        let numbered = !name.ends_with('*');
        let aligned = name.starts_with("align");
        // Per row: (cells of raw tokens, unnumbered flag, labels).
        type RawRow = (Vec<Vec<Token>>, bool, Vec<(String, Span)>);
        let mut rows: Vec<RawRow> = vec![(vec![Vec::new()], false, Vec::new())];
        let mut depth = 0usize;
        let mut end = open.end;
        let mut found_end = false;

        while self.i < self.t.len() {
            if self.expand_current_macro() {
                continue;
            }
            if depth == 0 {
                if let Some((after, end_span)) = environment_end_at(&self.t, self.i, name) {
                    self.i = after;
                    end = end_span.end;
                    found_end = true;
                    break;
                }
            }
            let token = self.t[self.i].token.clone();
            let row = rows.last_mut().expect("at least one row");
            match &token.kind {
                TokenKind::Command(command) if command == "label" => {
                    self.i += 1;
                    let (tokens, argument_span) = self.required_group("label", token.span);
                    let key = token_text(&tokens).trim().to_string();
                    if !key.is_empty() {
                        let row = rows.last_mut().expect("at least one row");
                        row.2.push((key, token.span.merge(argument_span)));
                    }
                    continue;
                }
                TokenKind::Command(command) if command == "nonumber" || command == "notag" => {
                    row.1 = true;
                }
                TokenKind::LineBreak if depth == 0 => {
                    rows.push((vec![Vec::new()], false, Vec::new()));
                }
                TokenKind::Word(word) if depth == 0 && word.contains('&') => {
                    let exact = token.span.end - token.span.start == word.len();
                    for (index, piece) in word.split('&').enumerate() {
                        if index > 0 {
                            row.0.push(Vec::new());
                        }
                        if piece.is_empty() {
                            continue;
                        }
                        let offset = piece.as_ptr() as usize - word.as_ptr() as usize;
                        let span = if exact {
                            Span::in_document(
                                token.span.document,
                                token.span.start + offset,
                                token.span.start + offset + piece.len(),
                            )
                        } else {
                            token.span
                        };
                        row.0.last_mut().expect("at least one cell").push(Token {
                            kind: TokenKind::Word(piece.to_string()),
                            span,
                        });
                    }
                }
                _ => {
                    // Nested groups and environments (`cases`, `pmatrix`)
                    // own their `\\` and `&`.
                    match &token.kind {
                        TokenKind::LBrace => depth += 1,
                        TokenKind::Command(command) if command == "begin" => depth += 1,
                        TokenKind::RBrace => depth = depth.saturating_sub(1),
                        TokenKind::Command(command) if command == "end" => {
                            depth = depth.saturating_sub(1)
                        }
                        _ => {}
                    }
                    row.0
                        .last_mut()
                        .expect("at least one cell")
                        .push(token.clone());
                }
            }
            end = token.span.end;
            self.i += 1;
        }
        if !found_end {
            self.diags.push(Diagnostic::error(
                format!("unterminated environment '{name}' — no matching \\end"),
                Some(open),
                Some("closed the display at end of input".into()),
            ));
        }
        // A trailing `\\` before `\end` does not start a real row.
        if rows.len() > 1
            && rows.last().is_some_and(|(cells, _, labels)| {
                labels.is_empty()
                    && cells.iter().flatten().all(|t| {
                        matches!(
                            t.kind,
                            TokenKind::Space | TokenKind::Comment | TokenKind::ParBreak
                        )
                    })
            })
        {
            rows.pop();
        }

        let mut math_rows = Vec::new();
        let mut labels = Vec::new();
        for (cells, unnumbered, row_labels) in rows {
            let span = cells
                .iter()
                .flatten()
                .map(|t| t.span)
                .reduce(Span::merge)
                .unwrap_or(open);
            let number = (numbered && !unnumbered).then(|| {
                self.equation_counter += 1;
                let number = self.equation_counter.to_string();
                self.current_counter = Some(number.clone());
                number
            });
            for (key, label_span) in row_labels {
                self.document_global_state = true;
                if self.seen_labels.insert(key.clone(), label_span).is_some() {
                    self.diags.push(Diagnostic::warning(
                        format!("duplicate \\label{{{key}}}; the second definition wins"),
                        Some(label_span),
                        Some("replaced the earlier label definition".into()),
                    ));
                }
                labels.push(Inline::Label {
                    key,
                    value: number
                        .clone()
                        .unwrap_or_else(|| self.equation_counter.to_string()),
                    span: label_span,
                });
            }
            let cells = cells
                .iter()
                .map(|cell| math::parse_tokens(cell, &mut self.diags))
                .collect();
            math_rows.push(MathRow {
                cells,
                number,
                span,
            });
        }
        para.push(Inline::MathRows {
            rows: math_rows,
            aligned,
            span: Span::in_document(open.document, open.start, end),
        });
        para.extend(labels);
        self.flush_paragraph(blocks, para);
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
        let invocation_start = self.i;
        self.i += 1;
        self.expand_macro(
            name,
            input.token.span,
            input.expansion_depth,
            definition,
            invocation_start,
        );
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
        // `\[...\]` and `$$...$$` are unnumbered displays in LaTeX: they never
        // print a number or advance the equation counter.
        para.push(Inline::Math {
            list,
            display,
            number: None,
            number_span: None,
            span: Span::in_document(open.document, open.start, end),
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
                        return (content, Span::in_document(open.document, open.start, end));
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
        (
            self.t[start..].to_vec(),
            Span::in_document(open.document, open.start, end),
        )
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
        let document = first.span.document;
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
        let span = Span::in_document(document, start, end);
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

    fn take_optional_star(&mut self) -> bool {
        self.skip_spaces();
        if matches!(
            self.peek().map(|token| &token.kind),
            Some(TokenKind::Word(word)) if word == "*"
        ) {
            self.i += 1;
            true
        } else {
            false
        }
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
            let content = std::mem::take(paragraph);
            blocks.push(match self.paragraph_styles.last() {
                Some(&style) => Block::Styled { style, content },
                None => Block::Paragraph(content),
            });
            self.finish_block_dependencies();
        }
    }

    /// Drops a `[<length>]` that directly follows `\\`, keeping any text glued
    /// to it (`\\[3pt]Next`) as the remainder of the word.
    fn skip_line_break_length(&mut self) {
        let Some(input) = self.t.get_mut(self.i) else {
            return;
        };
        let TokenKind::Word(word) = &input.token.kind else {
            return;
        };
        if !word.starts_with('[') {
            return;
        }
        let Some(close) = word.find(']') else {
            return;
        };
        let rest = word[close + 1..].to_string();
        if rest.is_empty() {
            self.i += 1;
            return;
        }
        let span = input.token.span;
        if span.end - span.start == word.len() {
            input.token.span = Span::in_document(span.document, span.start + close + 1, span.end);
        }
        input.token.kind = TokenKind::Word(rest);
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

/// True when loading `package` with `options` changes nothing about the output,
/// because the fixed layout already behaves that way.
fn package_matches_layout(package: &str, options: &str) -> bool {
    let options: Vec<&str> = options
        .split(',')
        .map(str::trim)
        .filter(|option| !option.is_empty())
        .collect();
    match package {
        // Source text is decoded as UTF-8 already.
        "inputenc" => options.iter().all(|option| *option == "utf8"),
        // Text glyphs are mapped from Unicode, which is what T1 approximates.
        "fontenc" => options.iter().all(|option| *option == "T1"),
        // Enumerate label templates are implemented; \setlist reports its own gap.
        "enumitem" => options.iter().all(|option| *option == "shortlabels"),
        "geometry" => {
            !options.is_empty()
                && options.iter().all(|option| match option.split_once('=') {
                    Some(("margin", value)) => length_pt(value)
                        .is_some_and(|pt| (pt - crate::layout::MARGIN_PT).abs() < 0.01),
                    None => *option == "letterpaper",
                    _ => false,
                })
        }
        // amsmath/amssymb/amsthm (math typesetting: \mathbb, \forall, gather,
        // align, ...) and microtype (character protrusion/expansion kerning)
        // are genuinely unimplemented and change real output; they must keep
        // warning rather than being silently matched here.
        _ => false,
    }
}

fn length_pt(value: &str) -> Option<f64> {
    let value = value.trim();
    let split = value
        .find(|c: char| c.is_ascii_alphabetic())
        .unwrap_or(value.len());
    let number: f64 = value[..split].trim().parse().ok()?;
    let per_unit = match value[split..].trim() {
        "in" => 72.0,
        "pt" => 72.0 / 72.27,
        "bp" => 1.0,
        "cm" => 72.0 / 2.54,
        "mm" => 72.0 / 25.4,
        _ => return None,
    };
    Some(number * per_unit)
}

/// Formats an enumitem label: a `label=` key using `\alph*`-style counters,
/// or a shortlabels template whose first `a A i I 1` is the counter.
fn enumitem_label(template: &str, count: u32) -> String {
    let counter = |style: char| match style {
        'a' => alphabetic(count, b'a'),
        'A' => alphabetic(count, b'A'),
        'i' => roman(count),
        'I' => roman(count).to_uppercase(),
        _ => count.to_string(),
    };
    if template.contains('=') {
        let Some(label) = template
            .split(',')
            .find_map(|key| key.trim().strip_prefix("label="))
        else {
            return format!("{}.", count);
        };
        return [
            ("\\alph*", 'a'),
            ("\\Alph*", 'A'),
            ("\\roman*", 'i'),
            ("\\Roman*", 'I'),
            ("\\arabic*", '1'),
        ]
        .iter()
        .fold(label.trim().to_string(), |text, (command, style)| {
            text.replace(command, &counter(*style))
        });
    }
    match template.char_indices().find(|(_, c)| "aAiI1".contains(*c)) {
        Some((index, style)) => format!(
            "{}{}{}",
            &template[..index],
            counter(style),
            &template[index + style.len_utf8()..]
        ),
        None => template.to_string(),
    }
}

fn alphabetic(count: u32, base: u8) -> String {
    match count {
        1..=26 => char::from(base + (count - 1) as u8).to_string(),
        _ => count.to_string(),
    }
}

fn roman(mut count: u32) -> String {
    const NUMERALS: &[(u32, &str)] = &[
        (1000, "m"),
        (900, "cm"),
        (500, "d"),
        (400, "cd"),
        (100, "c"),
        (90, "xc"),
        (50, "l"),
        (40, "xl"),
        (10, "x"),
        (9, "ix"),
        (5, "v"),
        (4, "iv"),
        (1, "i"),
    ];
    let mut text = String::new();
    for (value, numeral) in NUMERALS {
        while count >= *value {
            text.push_str(numeral);
            count -= value;
        }
    }
    text
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

fn paragraph_style(environment: &str) -> Option<ParagraphStyle> {
    match environment {
        "center" => Some(ParagraphStyle::Center),
        "flushright" => Some(ParagraphStyle::FlushRight),
        "flushleft" => Some(ParagraphStyle::FlushLeft),
        "quote" | "quotation" => Some(ParagraphStyle::Quote),
        _ => None,
    }
}

fn environment_end_at(
    tokens: &[InputToken],
    index: usize,
    expected: &str,
) -> Option<(usize, Span)> {
    let command = tokens.get(index)?;
    if !matches!(&command.token.kind, TokenKind::Command(name) if name == "end") {
        return None;
    }
    let mut cursor = index + 1;
    while matches!(
        tokens.get(cursor).map(|input| &input.token.kind),
        Some(TokenKind::Space | TokenKind::Comment)
    ) {
        cursor += 1;
    }
    if !matches!(
        tokens.get(cursor).map(|input| &input.token.kind),
        Some(TokenKind::LBrace)
    ) {
        return None;
    }
    cursor += 1;
    let name = tokens.get(cursor)?;
    if !matches!(&name.token.kind, TokenKind::Word(name) if name == expected) {
        return None;
    }
    cursor += 1;
    let close = tokens.get(cursor)?;
    if close.token.kind != TokenKind::RBrace {
        return None;
    }
    Some((cursor + 1, command.token.span.merge(close.token.span)))
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

    fn pages(source: &str) -> (Parsed, Vec<crate::layout::Page>) {
        let parsed = parse(source);
        let pages = layout::layout(&parsed.blocks);
        (parsed, pages)
    }

    #[test]
    fn dimen_parsing_supports_the_common_units() {
        assert_eq!(parse_dimen_pt("12pt"), Some(12.0));
        assert_eq!(parse_dimen_pt(" 1em "), Some(crate::layout::BODY_SIZE_PT));
        assert_eq!(parse_dimen_pt("1in"), Some(72.27));
        assert!(parse_dimen_pt("banana").is_none());
        assert!(parse_dimen_pt("").is_none());
    }

    #[test]
    fn newpage_forces_a_fresh_page_even_with_room_left() {
        let (parsed, pages) = pages(r"First page\newpage Second page");
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert_eq!(pages.len(), 2, "expected exactly one forced page break");
        assert!(pages[0].items.iter().any(|item| item.text == "First"));
        assert!(pages[1].items.iter().any(|item| item.text == "Second"));
    }

    #[test]
    fn hrule_emits_a_full_measure_rule_with_a_real_span() {
        let source = r"Above\hrule Below";
        let (parsed, items) = items(source);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let rule_item = items
            .iter()
            .find(|item| item.rule.is_some())
            .expect("hrule must emit an item carrying rule geometry");
        let rule = rule_item.rule.unwrap();
        assert!(rule.width_pt > 0.0);
        assert!(rule.height_pt > 0.0);
        assert_eq!(
            rule_item.span,
            Span::new(
                source.find("\\hrule").unwrap(),
                source.find("\\hrule").unwrap() + "\\hrule".len()
            )
        );
    }

    #[test]
    fn vspace_adds_extra_gap_beyond_the_ordinary_paragraph_gap() {
        let baseline = items("One\n\nTwo").1;
        let spaced = items(r"One\vspace{50pt}Two").1;
        let one = baseline.iter().find(|i| i.text == "One").unwrap();
        let two_baseline = baseline.iter().find(|i| i.text == "Two").unwrap();
        let two_spaced = spaced.iter().find(|i| i.text == "Two").unwrap();
        assert!(
            two_spaced.baseline_y_pt - one.baseline_y_pt
                > two_baseline.baseline_y_pt - one.baseline_y_pt,
            "\\vspace{{50pt}} should push the following text further down than an ordinary paragraph break"
        );
    }

    #[test]
    fn pagestyle_is_accepted_without_a_diagnostic() {
        for style in ["empty", "plain", "headings"] {
            let parsed = parse(&format!(r"\pagestyle{{{style}}}Body text"));
            assert!(
                parsed.diagnostics.is_empty(),
                "\\pagestyle{{{style}}}: {:?}",
                parsed.diagnostics
            );
        }
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
    fn starred_subsection_consumes_its_star_and_does_not_advance_numbering() {
        let parsed = parse(r"\section{One}\subsection*{Aside}\subsection{Two}");
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let numbers: Vec<&str> = parsed
            .blocks
            .iter()
            .filter_map(|block| match block {
                Block::Heading { number, .. } => Some(number.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(numbers, ["1", "", "1.1"]);
    }

    #[test]
    fn problem_style_macro_and_font_declarations_preserve_content_without_errors() {
        let source = r"\newcommand{\problem}[2]{\subsection*{Problem #1 \hfill \normalfont[#2 points]}}\problem{1}{4}{\bfseries Body}";
        let (parsed, items) = items(source);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert!(items.iter().any(|item| item.text == "Problem"));
        assert!(items.iter().any(|item| item.text == "Body"));
        assert!(!items.iter().any(|item| item.text == "*"));
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

    #[test]
    fn gather_star_rows_are_math_with_no_diagnostics() {
        let source = "\\documentclass{article}\\begin{document}\n\\begin{gather*}\\int_{0}^{\\infty} e^{-x^{2}}\\,dx = \\frac{\\sqrt{\\pi}}{2} \\\\ \\sum_{n=1}^{\\infty}\\frac{1}{n^{2}} = \\frac{\\pi^{2}}{6}\\end{gather*}\n\\end{document}";
        let (parsed, items) = items(source);
        let errors: Vec<_> = parsed
            .diagnostics
            .iter()
            .filter(|d| d.severity == crate::diagnostics::Severity::Error)
            .collect();
        assert!(errors.is_empty(), "{errors:?}");
        let Block::Paragraph(inlines) = &parsed.blocks[0] else {
            panic!("expected a paragraph");
        };
        let Inline::MathRows { rows, aligned, .. } = &inlines[0] else {
            panic!("expected multi-row math, got {inlines:?}");
        };
        assert!(!aligned);
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().all(|row| row.number.is_none()));
        let texts: Vec<_> = items.iter().map(|i| i.text.as_str()).collect();
        for glyph in ["∫", "∑", "π", "∞"] {
            assert!(texts.contains(&glyph), "{glyph} missing from {texts:?}");
        }
        assert!(!texts.contains(&","), "\\, must be spacing, not a comma");
        let int_y = items.iter().find(|i| i.text == "∫").unwrap().baseline_y_pt;
        let sum_y = items.iter().find(|i| i.text == "∑").unwrap().baseline_y_pt;
        assert!(sum_y > int_y, "second row must sit below the first");
    }

    #[test]
    fn align_shares_tab_stop_and_numbers_rows() {
        let source = "\\begin{align}x^{2} &= y \\label{a}\\\\ 2xyz &= 1 \\nonumber\\\\ w &= 3\\\\\\end{align}\\ref{a}";
        let (parsed, items) = items(source);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let equals: Vec<_> = items.iter().filter(|i| i.text == "=").collect();
        assert_eq!(equals.len(), 3);
        assert!(equals
            .iter()
            .all(|i| (i.x_pt - equals[0].x_pt).abs() < 0.01));
        let numbers: Vec<_> = items
            .iter()
            .filter(|i| i.text.starts_with('('))
            .map(|i| i.text.as_str())
            .collect();
        assert_eq!(numbers, ["(1)", "(2)"]);
        let Block::Paragraph(inlines) = &parsed.blocks[0] else {
            panic!("expected a paragraph");
        };
        assert!(inlines.iter().any(
            |inline| matches!(inline, Inline::Label { key, value, .. } if key == "a" && value == "1")
        ));
    }

    #[test]
    fn equation_star_is_unnumbered_display_math() {
        let (parsed, items) = items("\\begin{equation*}a=b\\end{equation*}");
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert_eq!(
            items.iter().map(|i| i.text.as_str()).collect::<Vec<_>>(),
            ["a", "=", "b"]
        );
    }

    #[test]
    fn math_grid_environments_lay_out_cells_in_rows_and_columns() {
        let source = "\\[ f = \\begin{cases} x & x \\geq 0 \\\\ -y & y < 0 \\end{cases} \\]\n\\begin{gather*}\\begin{pmatrix} 1 & 2 \\\\ 3 & 4 \\end{pmatrix}\\end{gather*}\n\\[\\begin{array}{rl} a & b,\\\\[2pt] cc & d \\end{array}\\]";
        let (parsed, items) = items(source);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let at = |text: &str| items.iter().find(|i| i.text == text).unwrap();
        // cases: left brace only, two rows, second column shared.
        assert!(items.iter().any(|i| i.text == "{"));
        assert!(at("≥").baseline_y_pt < at("<").baseline_y_pt);
        // pmatrix: fences and a 2x2 grid.
        assert!(items.iter().any(|i| i.text == "(") && items.iter().any(|i| i.text == ")"));
        assert_eq!(at("1").baseline_y_pt, at("2").baseline_y_pt);
        assert_eq!(at("1").x_pt, at("3").x_pt);
        assert!(at("3").baseline_y_pt > at("1").baseline_y_pt);
        // array {rl}: right-aligned first column, `[2pt]` consumed.
        assert!(!items.iter().any(|i| i.text == "p" || i.text == "t"));
        let a = at("a");
        let cs: Vec<_> = items.iter().filter(|i| i.text == "c").collect();
        assert!(a.x_pt > cs[0].x_pt, "right-aligned column");
        assert_eq!(at("b").x_pt, at("d").x_pt);
    }

    #[test]
    fn center_and_quote_align_their_paragraphs() {
        let source = "Plain.\n\\begin{center}Title\\\\[3pt]Subtitle words\\end{center}\n\\begin{quote}Quoted.\\end{quote}\nAfter.";
        let (parsed, items) = items(source);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert!(matches!(
            parsed.blocks[1],
            Block::Styled {
                style: ParagraphStyle::Center,
                ..
            }
        ));
        let at = |text: &str| items.iter().find(|i| i.text == text).unwrap();
        assert!(!items.iter().any(|i| i.text.contains("3pt")));
        let page_centre = crate::layout::PAGE_WIDTH_PT / 2.0;
        assert!((at("Title").x_pt - page_centre).abs() < 40.0);
        assert!(at("Subtitle").x_pt > crate::layout::MARGIN_PT + 100.0);
        assert_eq!(
            at("Quoted.").x_pt,
            crate::layout::MARGIN_PT + crate::layout::QUOTE_INDENT_PT
        );
        assert_eq!(at("After.").x_pt, crate::layout::MARGIN_PT);
        assert_eq!(at("Plain.").x_pt, crate::layout::MARGIN_PT);
    }

    #[test]
    fn only_numbered_displays_print_numbers_and_advance_the_counter() {
        let source = "\\[a\\] $$b$$ \\begin{displaymath}c\\end{displaymath}\\begin{equation*}d\\end{equation*}\\begin{gather*}e\\end{gather*}\\begin{align*}f&=g\\end{align*}\\begin{equation}h\\label{h}\\end{equation}\\begin{align}i\\nonumber\\\\j\\label{j}\\end{align}";
        let (parsed, items) = items(source);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let numbers: Vec<_> = items
            .iter()
            .filter(|i| i.text.starts_with('('))
            .map(|i| i.text.as_str())
            .collect();
        assert_eq!(numbers, ["(1)", "(2)"]);
        let labels: Vec<_> = parsed
            .blocks
            .iter()
            .flat_map(|block| match block {
                Block::Paragraph(inlines) => inlines.as_slice(),
                _ => &[],
            })
            .filter_map(|inline| match inline {
                Inline::Label { key, value, .. } => Some((key.as_str(), value.as_str())),
                _ => None,
            })
            .collect();
        assert_eq!(labels, [("h", "1"), ("j", "2")]);
    }
}
