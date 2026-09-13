//! Parsing for the LaTeX box commands; the node model is `crate::boxes`.
//!
//! Every content argument is parsed with the ordinary paragraph parser as
//! its own group (like a `tabular` entry), so text keeps its exact source
//! spans and nested commands, math and boxes work inside it. `\parbox` and
//! `minipage` keep their paragraphs and alignment declarations;
//! `@parboxrestore` resets the surrounding list and alignment state, so none
//! of it leaks in.

use super::{environment_end_at, environment_name_at, Block, Inline, InputToken, P};
use crate::boxes::{
    BoxDimen, BoxParagraph, BoxUnit, LengthAssignment, LengthValue, MeasuredDimension, TextBox,
    TextBoxKind,
};
use crate::diagnostics::Diagnostic;
use crate::lexer::{Token, TokenKind};
use crate::Span;
use std::collections::HashMap;

/// Kernel lengths a box dimension may name without `\newlength`.
const KERNEL_LENGTHS: &[&str] = &[
    "textwidth",
    "linewidth",
    "columnwidth",
    "hsize",
    "textheight",
    "fboxsep",
    "fboxrule",
    "parindent",
    "parskip",
    "baselineskip",
    "tabcolsep",
    "arrayrulewidth",
    "labelwidth",
    "labelsep",
    "leftmargin",
    "marginparwidth",
    "unitlength",
];

/// A dimension argument's source text, keeping control words (`token_text`
/// drops their backslash).
fn dimen_source(tokens: &[InputToken]) -> String {
    let mut out = String::new();
    for input in tokens {
        match &input.token.kind {
            TokenKind::Word(text) => out.push_str(text),
            TokenKind::Command(name) => {
                out.push('\\');
                out.push_str(name);
            }
            TokenKind::Space | TokenKind::ParBreak => out.push(' '),
            _ => {}
        }
    }
    out
}

impl P<'_> {
    /// `[...]` after a box command. Unlike `optional_bracket_argument`, a
    /// word such as `[2cm][r]` is split at the first top-level `]` and the
    /// rest stays in the stream for the next argument.
    fn box_optional(&mut self) -> Option<(String, Span)> {
        self.skip_spaces();
        let first = self.peek()?;
        let TokenKind::Word(word) = &first.kind else {
            return None;
        };
        if !word.starts_with('[') {
            return None;
        }
        let start = first.span;
        let mut text = String::new();
        let mut depth = 0usize;
        let mut opened = false;
        while self.i < self.t.len() {
            let token = self.t[self.i].token.clone();
            match &token.kind {
                TokenKind::Word(word) => {
                    for (offset, ch) in word.char_indices() {
                        match ch {
                            '[' if !opened => {
                                opened = true;
                                depth = 1;
                                continue;
                            }
                            '[' => depth += 1,
                            ']' if depth == 1 => {
                                let rest = &word[offset + 1..];
                                let close = token.span.start + offset + 1;
                                if rest.is_empty() {
                                    self.i += 1;
                                } else {
                                    // Leave the rest of the word for the next argument.
                                    self.t[self.i].token = Token {
                                        kind: TokenKind::Word(rest.to_string()),
                                        span: Span::in_document(
                                            token.span.document,
                                            close.min(token.span.end),
                                            token.span.end,
                                        ),
                                    };
                                }
                                let span = Span::in_document(start.document, start.start, close.min(token.span.end));
                                return Some((text, span));
                            }
                            ']' => depth -= 1,
                            _ => {}
                        }
                        text.push(ch);
                    }
                }
                TokenKind::Command(name) => {
                    text.push('\\');
                    text.push_str(name);
                }
                TokenKind::Space => text.push(' '),
                TokenKind::LBrace => text.push('{'),
                TokenKind::RBrace => text.push('}'),
                _ => break,
            }
            self.i += 1;
            if depth == 0 {
                break;
            }
        }
        let span = start.merge(self.t.get(self.i.saturating_sub(1)).map_or(start, |t| t.token.span));
        self.diags.push(Diagnostic::error(
            "optional argument is missing its closing ']'",
            Some(span),
            Some("used the text through the end of the argument as the option".into()),
        ));
        Some((text, span))
    }

    /// A dimension argument (`{2cm}`, `[.5\textwidth]`), with a diagnostic
    /// naming `command` when it is not one.
    fn box_dimen(&mut self, command: &str, raw: &str, span: Span) -> Option<BoxDimen> {
        match BoxDimen::parse(raw) {
            Some(dimen) => {
                if let BoxUnit::Length(name) = &dimen.unit {
                    if !self.lengths.contains(name) && !KERNEL_LENGTHS.contains(&name.as_str()) {
                        self.diags.push(Diagnostic::error(
                            format!("\\{command}: \\{name} is not a length (declare it with \\newlength)"),
                            Some(span),
                            Some("used 0pt for the dimension".into()),
                        ));
                    }
                }
                Some(dimen)
            }
            None => {
                self.diags.push(Diagnostic::error(
                    format!("\\{command} requires a dimension, got '{}'", raw.trim()),
                    Some(span),
                    Some("used 0pt for the dimension".into()),
                ));
                None
            }
        }
    }

    /// `\hspace{\w}`/`\hspace{.5\textwidth}`: a dimension naming a length.
    pub(super) fn length_dimen(&self, tokens: &[InputToken]) -> Option<BoxDimen> {
        let dimen = BoxDimen::parse(&dimen_source(tokens))?;
        match &dimen.unit {
            BoxUnit::Length(name) if self.lengths.contains(name) || KERNEL_LENGTHS.contains(&name.as_str()) => Some(dimen),
            _ => None,
        }
    }

    /// `\setlength{\len}{..}` of a `\newlength` in the body, or of
    /// `\fboxsep`/`\fboxrule` anywhere (the typesetter reads those from the
    /// source). Returns false when the assignment is not a box length.
    pub(super) fn set_box_length(&mut self, target: &str, value: &[InputToken], span: Span, para: &mut Vec<Inline>) -> bool {
        let raw = dimen_source(value);
        if matches!(target, "fboxsep" | "fboxrule") {
            if BoxDimen::parse(&raw).is_none() {
                self.diags.push(Diagnostic::error(
                    format!("\\setlength{{\\{target}}} requires a dimension, got '{}'", raw.trim()),
                    Some(span),
                    Some("ignored the length assignment".into()),
                ));
            }
            return true;
        }
        if !self.lengths.contains(target) {
            return false;
        }
        if let Some(dimen) = self.box_dimen("setlength", &raw, span) {
            if self.in_body {
                para.push(Inline::SetLength(Box::new(LengthAssignment {
                    name: target.to_string(),
                    value: LengthValue::Dimen(dimen),
                    span,
                })));
            }
        }
        true
    }

    /// Parses `tokens` as a group of its own and returns its blocks. List,
    /// alignment and `\item` state of the surrounding text is hidden, as
    /// `\@parboxrestore` does; the current font style carries in.
    fn box_blocks(&mut self, tokens: Vec<InputToken>) -> Vec<Block> {
        let outer_tokens = std::mem::replace(&mut self.t, tokens);
        let outer_index = std::mem::replace(&mut self.i, 0);
        let outer_style = self.style;
        let outer_label = self.pending_item_label.take();
        let outer_lists = std::mem::take(&mut self.list_stack);
        let outer_paragraph_styles = std::mem::take(&mut self.paragraph_styles);
        let outer_alignment = self.declared_alignment.take();
        let outer_braces = self.brace_stack.len();
        let outer_style_stack = self.style_stack.len();
        let outer_alignment_stack = self.alignment_stack.len();
        let outer_dependency_blocks = self.block_dependencies.len();
        self.macro_scopes.push(HashMap::new());
        let mut blocks = Vec::new();
        let mut para = Vec::new();
        self.parse_stream(&mut blocks, &mut para);
        self.flush_paragraph(&mut blocks, &mut para);
        while self.brace_stack.len() > outer_braces {
            self.brace_stack.pop();
            self.restore_scope();
        }
        self.style_stack.truncate(outer_style_stack);
        self.alignment_stack.truncate(outer_alignment_stack);
        self.restore_scope();
        self.block_dependencies.truncate(outer_dependency_blocks);
        self.t = outer_tokens;
        self.i = outer_index;
        self.style = outer_style;
        self.pending_item_label = outer_label;
        self.list_stack = outer_lists;
        self.paragraph_styles = outer_paragraph_styles;
        self.declared_alignment = outer_alignment;
        blocks
    }

    /// Horizontal box content: `\par` does nothing in restricted
    /// horizontal mode, so paragraphs run together.
    fn box_inlines(&mut self, tokens: Vec<InputToken>, command: &str, span: Span) -> Vec<Inline> {
        let mut content = Vec::new();
        for block in self.box_blocks(tokens) {
            match block {
                Block::Paragraph(inlines) | Block::Styled { content: inlines, .. } => content.extend(inlines),
                other => self.box_block_limitation(command, &other, span, &mut content),
            }
        }
        content
    }

    /// `\parbox`/`minipage` content: one [`BoxParagraph`] per paragraph.
    fn box_paragraphs(&mut self, tokens: Vec<InputToken>, command: &str, span: Span) -> Vec<BoxParagraph> {
        let mut paragraphs = Vec::new();
        for block in self.box_blocks(tokens) {
            match block {
                Block::Paragraph(content) => paragraphs.push(BoxParagraph { style: None, content }),
                Block::Styled { style, content } => paragraphs.push(BoxParagraph { style: Some(style), content }),
                other => {
                    let mut content = Vec::new();
                    self.box_block_limitation(command, &other, span, &mut content);
                    if !content.is_empty() {
                        paragraphs.push(BoxParagraph { style: None, content });
                    }
                }
            }
        }
        paragraphs
    }

    /// A block a box cannot hold as such: its text is kept as a plain
    /// paragraph (lists, headings) or dropped (vertical glue, rules), with a
    /// warning either way.
    fn box_block_limitation(&mut self, command: &str, block: &Block, span: Span, content: &mut Vec<Inline>) {
        let (kind, inlines): (&str, Option<&Vec<Inline>>) = match block {
            Block::ListItem { content, .. } => ("a list item", Some(content)),
            Block::Heading { content, .. } => ("a heading", Some(content)),
            Block::FigureCaption { content } => ("a caption", Some(content)),
            Block::VSpace { .. } | Block::VFill => ("vertical space", None),
            Block::Rule { .. } => ("a rule", None),
            Block::PageBreak => ("a page break", None),
            Block::Verbatim { .. } => ("verbatim text", None),
            Block::TableOfContents { .. } => ("a table of contents", None),
            Block::TitleBlock { .. } => ("a title block", None),
            Block::Paragraph(_) | Block::Styled { .. } => return,
        };
        self.diags.push(Diagnostic::warning(
            format!("{kind} inside \\{command} is not laid out as such"),
            Some(span),
            Some(if inlines.is_some() {
                "set its text as a plain paragraph of the box".into()
            } else {
                "left it out of the box".into()
            }),
        ));
        if let Some(inlines) = inlines {
            content.extend(inlines.iter().cloned());
        }
    }

    fn single_position(&mut self, command: &str, raw: Option<(String, Span)>, allowed: &str) -> Option<char> {
        let (raw, span) = raw?;
        let trimmed = raw.trim();
        let mut chars = trimmed.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) if allowed.contains(c) => Some(c),
            (None, _) => None,
            _ => {
                self.diags.push(Diagnostic::warning(
                    format!("\\{command}: unexpected alignment '{trimmed}'"),
                    Some(span),
                    Some("centred the content, as LaTeX does".into()),
                ));
                Some('c')
            }
        }
    }

    /// A `{\name}` argument naming a box or length register.
    fn register_name(&mut self, command: &str, span: Span) -> (Option<String>, Span) {
        let (tokens, argument_span) = self.required_group(command, span);
        let names: Vec<&str> = tokens
            .iter()
            .filter_map(|t| match &t.token.kind {
                TokenKind::Command(name) => Some(name.as_str()),
                TokenKind::Space | TokenKind::Comment => None,
                _ => Some(""),
            })
            .collect();
        match names.as_slice() {
            [name] if !name.is_empty() => (Some((*name).to_string()), argument_span),
            _ => {
                self.diags.push(Diagnostic::error(
                    format!("\\{command} requires a single command name, got '{}'", dimen_source(&tokens).trim()),
                    Some(span.merge(argument_span)),
                    Some("ignored the command".into()),
                ));
                (None, argument_span)
            }
        }
    }

    pub(super) fn box_command(&mut self, name: &str, span: Span, para: &mut Vec<Inline>) {
        let space_before = self.space_precedes(self.i - 1);
        let mut end = span;
        let mut paragraphs = Vec::new();
        let (kind, content) = match name {
            "strut" => (TextBoxKind::Strut, Vec::new()),
            "usebox" => {
                let (register, argument_span) = self.register_name(name, span);
                let full = span.merge(argument_span);
                let Some(register) = register else { return };
                match self.saved_boxes.get(&register) {
                    Some(Some(saved)) => {
                        let mut reused = saved.clone();
                        reused.span = full;
                        reused.space_before = space_before;
                        para.push(Inline::Box(Box::new(reused)));
                    }
                    Some(None) => {
                        // `\usebox` of a void register appends nothing.
                    }
                    None => self.diags.push(Diagnostic::error(
                        format!("\\usebox: \\{register} is not a box (declare it with \\newsavebox)"),
                        Some(full),
                        Some("typeset nothing for the box".into()),
                    )),
                }
                return;
            }
            "mbox" | "phantom" | "hphantom" | "vphantom" | "smash" | "llap" | "rlap" => {
                let (tokens, argument_span) = self.long_required_group(name, span);
                end = argument_span;
                let content = self.box_inlines(tokens, name, span.merge(argument_span));
                let kind = match name {
                    "mbox" => TextBoxKind::Make { width: None, pos: None, frame: false },
                    "phantom" => TextBoxKind::Phantom { horizontal: true, vertical: true },
                    "hphantom" => TextBoxKind::Phantom { horizontal: true, vertical: false },
                    "vphantom" => TextBoxKind::Phantom { horizontal: false, vertical: true },
                    "smash" => TextBoxKind::Smash,
                    "llap" => TextBoxKind::Lap { left: true },
                    _ => TextBoxKind::Lap { left: false },
                };
                (kind, content)
            }
            "fbox" => {
                let (tokens, argument_span) = self.long_required_group(name, span);
                end = argument_span;
                let content = self.box_inlines(tokens, name, span.merge(argument_span));
                (TextBoxKind::Make { width: None, pos: None, frame: true }, content)
            }
            "makebox" | "framebox" => {
                let (kind, content, last) = self.make_box_arguments(name, span);
                end = last;
                (kind, content)
            }
            "raisebox" => {
                let (lift_tokens, lift_span) = self.required_group(name, span);
                let lift = self.box_dimen(name, &dimen_source(&lift_tokens), lift_span).unwrap_or_else(zero);
                let height = self.box_optional().and_then(|(raw, s)| self.box_dimen(name, &raw, s));
                let depth = if height.is_some() {
                    self.box_optional().and_then(|(raw, s)| self.box_dimen(name, &raw, s))
                } else {
                    None
                };
                let (tokens, argument_span) = self.long_required_group(name, span);
                end = argument_span;
                let content = self.box_inlines(tokens, name, span.merge(argument_span));
                (TextBoxKind::Raise { lift, height, depth }, content)
            }
            "parbox" => {
                let pos = self.box_optional();
                let pos = self.single_position(name, pos, "tcbs");
                let height = self.box_optional().and_then(|(raw, s)| self.box_dimen(name, &raw, s));
                let inner = self.box_optional();
                let inner = self.single_position(name, inner, "tcbs");
                let (width_tokens, width_span) = self.required_group(name, span);
                let width = self.box_dimen(name, &dimen_source(&width_tokens), width_span).unwrap_or_else(zero);
                let (tokens, argument_span) = self.long_required_group(name, span);
                end = argument_span;
                paragraphs = self.box_paragraphs(tokens, name, span.merge(argument_span));
                (TextBoxKind::Par { pos, height, inner, width, minipage: false }, Vec::new())
            }
            _ => unreachable!("dispatched box command"),
        };
        para.push(Inline::Box(Box::new(TextBox {
            kind,
            content,
            paragraphs,
            span: span.merge(end),
            space_before,
        })));
    }

    /// `\makebox[w][pos]{..}` / `\framebox[w][pos]{..}` (and `\savebox`'s
    /// tail): the kind, the content and the last argument's span.
    fn make_box_arguments(&mut self, name: &str, span: Span) -> (TextBoxKind, Vec<Inline>, Span) {
        let width_raw = self.box_optional();
        let width = width_raw.and_then(|(raw, s)| self.box_dimen(name, &raw, s));
        let pos = if width.is_some() {
            let pos = self.box_optional();
            self.single_position(name, pos, "lcrs")
        } else {
            None
        };
        let (tokens, argument_span) = self.long_required_group(name, span);
        let content = self.box_inlines(tokens, name, span.merge(argument_span));
        let frame = name == "framebox";
        (TextBoxKind::Make { width, pos, frame }, content, argument_span)
    }

    pub(super) fn box_register_command(&mut self, name: &str, span: Span, para: &mut Vec<Inline>) {
        match name {
            "newsavebox" => {
                let (register, argument_span) = self.register_name(name, span);
                if let Some(register) = register {
                    if self.saved_boxes.contains_key(&register) {
                        self.diags.push(Diagnostic::error(
                            format!("\\newsavebox: \\{register} is already defined"),
                            Some(span.merge(argument_span)),
                            Some("kept the existing box".into()),
                        ));
                    } else {
                        self.saved_boxes.insert(register, None);
                    }
                }
            }
            "newlength" => {
                let (register, argument_span) = self.register_name(name, span);
                if let Some(register) = register {
                    if !self.lengths.insert(register.clone()) {
                        self.diags.push(Diagnostic::error(
                            format!("\\newlength: \\{register} is already defined"),
                            Some(span.merge(argument_span)),
                            Some("kept the existing length".into()),
                        ));
                    }
                }
            }
            "sbox" | "savebox" => {
                let (register, register_span) = self.register_name(name, span);
                let (kind, content, last) = if name == "sbox" {
                    let (tokens, argument_span) = self.long_required_group(name, span);
                    let content = self.box_inlines(tokens, name, span.merge(argument_span));
                    (TextBoxKind::Make { width: None, pos: None, frame: false }, content, argument_span)
                } else {
                    let (kind, content, last) = self.make_box_arguments(name, span);
                    // `\savebox` never frames.
                    let kind = match kind {
                        TextBoxKind::Make { width, pos, .. } => TextBoxKind::Make { width, pos, frame: false },
                        other => other,
                    };
                    (kind, content, last)
                };
                let full = span.merge(last);
                self.store_box(register, register_span, name, TextBox { kind, content, paragraphs: Vec::new(), span: full, space_before: false });
            }
            "settowidth" | "settoheight" | "settodepth" => {
                let (register, register_span) = self.register_name(name, span);
                let (tokens, argument_span) = self.long_required_group(name, span);
                let full = span.merge(argument_span);
                let content = self.box_inlines(tokens, name, full);
                let Some(register) = register else { return };
                if !self.lengths.contains(&register) && !KERNEL_LENGTHS.contains(&register.as_str()) {
                    self.diags.push(Diagnostic::error(
                        format!("\\{name}: \\{register} is not a length (declare it with \\newlength)"),
                        Some(span.merge(register_span)),
                        Some("measured the content but assigned nothing".into()),
                    ));
                    return;
                }
                let which = match name {
                    "settowidth" => MeasuredDimension::Width,
                    "settoheight" => MeasuredDimension::Height,
                    _ => MeasuredDimension::Depth,
                };
                if self.in_body {
                    para.push(Inline::SetLength(Box::new(LengthAssignment {
                        name: register,
                        value: LengthValue::Measure { which, content },
                        span: full,
                    })));
                }
            }
            _ => unreachable!("dispatched box register command"),
        }
    }

    fn store_box(&mut self, register: Option<String>, register_span: Span, command: &str, b: TextBox) {
        let Some(register) = register else { return };
        match self.saved_boxes.get_mut(&register) {
            Some(slot) => *slot = Some(b),
            None => self.diags.push(Diagnostic::error(
                format!("\\{command}: \\{register} is not a box (declare it with \\newsavebox)"),
                Some(register_span),
                Some("discarded the box content".into()),
            )),
        }
    }

    /// The body of `\begin{name}` up to its matching `\end{name}`, and the
    /// span of that `\end{name}` (the end of input when it is missing).
    fn environment_body(&mut self, name: &str, open: Span) -> (Vec<InputToken>, Span) {
        let start = self.i;
        let mut depth = 0usize;
        while self.i < self.t.len() {
            if let Some((after, end_span)) = environment_end_at(&self.t, self.i, name) {
                if depth == 0 {
                    let body = self.t[start..self.i].to_vec();
                    self.i = after;
                    return (body, end_span);
                }
                depth -= 1;
                self.i = after;
                continue;
            }
            if matches!(&self.t[self.i].token.kind, TokenKind::Command(c) if c == "begin")
                && environment_name_at(&self.t, self.i) == Some(name)
            {
                depth += 1;
            }
            self.i += 1;
        }
        self.diags.push(Diagnostic::error(
            format!("\\begin{{{name}}} is never closed"),
            Some(open),
            Some("closed the environment at the end of input".into()),
        ));
        let last = self.t.last().map_or(open, |t| t.token.span);
        (self.t[start..].to_vec(), last)
    }

    /// `minipage` and `lrbox`.
    pub(super) fn box_environment(&mut self, open: Span, name: &str, para: &mut Vec<Inline>) {
        let space_before = self.t[..self.i]
            .iter()
            .rposition(|input| input.token.span == open)
            .is_none_or(|index| super::preceded_by_space(&self.t, index));
        if name == "lrbox" {
            let (register, register_span) = self.register_name(name, open);
            let (body, end) = self.environment_body(name, open);
            let full = open.merge(end);
            let content = self.box_inlines(body, name, full);
            self.store_box(
                register,
                register_span,
                name,
                TextBox {
                    kind: TextBoxKind::Make { width: None, pos: None, frame: false },
                    content,
                    paragraphs: Vec::new(),
                    span: full,
                    space_before: false,
                },
            );
            return;
        }
        let pos = self.box_optional();
        let pos = self.single_position(name, pos, "tcb");
        let height = self.box_optional().and_then(|(raw, s)| self.box_dimen(name, &raw, s));
        let inner = self.box_optional();
        let inner = self.single_position(name, inner, "tcbs");
        let (width_tokens, width_span) = self.required_group(name, open);
        let width = self.box_dimen(name, &dimen_source(&width_tokens), width_span).unwrap_or_else(zero);
        let (body, end) = self.environment_body(name, open);
        let full = open.merge(end);
        let paragraphs = self.box_paragraphs(body, name, full);
        para.push(Inline::Box(Box::new(TextBox {
            kind: TextBoxKind::Par { pos, height, inner, width, minipage: true },
            content: Vec::new(),
            paragraphs,
            span: full,
            space_before,
        })));
    }
}

fn zero() -> BoxDimen {
    BoxDimen { negative: false, integer: 0, frac: Vec::new(), unit: BoxUnit::Physical("pt".into()) }
}
