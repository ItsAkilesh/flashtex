//! Parsing for `tabular`/`tabular*`; the alignment model and layout live in
//! `crate::tabular`.
//!
//! The body is split into rows on top-level `\\` (and `\tabularnewline`) and
//! into entries on top-level `&`, the way `\halign` sees them: braces and
//! nested environments own their own `\\` and `&`. Each entry is then parsed
//! with the ordinary paragraph parser as its own group, so every text item
//! keeps its exact source span. Rules (`\hline`, `\cline`, booktabs) are only
//! recognised where TeX allows `\noalign`: at the start of a row.

use super::{
    environment_end_at, parse_dimen_pt_at, preceded_by_space, token_text, Block, Inline,
    InputToken, P,
};
use crate::diagnostics::Diagnostic;
use crate::lexer::{Token, TokenKind};
use crate::tabular::{
    Align, BookRule, Cell, ColumnTemplate, Entry, Length, Material, Row, Tabular, VerticalPosition,
    ARRAYRULEWIDTH_PT, DOUBLERULESEP_PT, TABCOLSEP_PT,
};
use crate::Span;

/// Bound on `*{n}{...}` expansion, far beyond any real column specification.
const MAX_SPEC_ITEMS: usize = 4096;
const MAX_SPEC_DEPTH: usize = 8;

enum SpecItem {
    Char(char, Span),
    Group(Vec<InputToken>, Span),
    Command(String, Span),
}

struct RawCell {
    tokens: Vec<InputToken>,
    /// Column count, specification tokens and the `\multicolumn` span.
    multicolumn: Option<(usize, Vec<InputToken>, Span)>,
    /// The `&` that opened this entry.
    amp: Option<Span>,
}

impl RawCell {
    fn new(amp: Option<Span>) -> Self {
        RawCell {
            tokens: Vec::new(),
            multicolumn: None,
            amp,
        }
    }

    fn is_blank(&self) -> bool {
        self.multicolumn.is_none() && blank(&self.tokens)
    }
}

enum RawEntry {
    /// Entries plus the `\\[<dimen>]` argument, if any.
    Row(Vec<RawCell>, Option<f64>),
    Done(Entry),
}

fn blank(tokens: &[InputToken]) -> bool {
    tokens.iter().all(|input| {
        matches!(
            input.token.kind,
            TokenKind::Space | TokenKind::Comment | TokenKind::ParBreak
        )
    })
}

fn row_is_blank(row: &[RawCell]) -> bool {
    row.len() == 1 && row[0].is_blank()
}

fn is_rule_command(name: &str, booktabs: bool) -> bool {
    matches!(name, "hline" | "cline")
        || (booktabs && matches!(name, "toprule" | "midrule" | "bottomrule" | "cmidrule"))
}

/// A `tabular*` width or `p{}` width: a dimension, or a multiple of the text
/// width.
fn table_length(tokens: &[InputToken], body: f64) -> Option<Length> {
    let significant: Vec<&InputToken> = tokens
        .iter()
        .filter(|input| !matches!(input.token.kind, TokenKind::Space | TokenKind::Comment))
        .collect();
    if let Some((last, factor)) = significant.split_last() {
        if let TokenKind::Command(name) = &last.token.kind {
            if matches!(
                name.as_str(),
                "textwidth" | "linewidth" | "columnwidth" | "hsize"
            ) {
                let mut text = String::new();
                for input in factor {
                    match &input.token.kind {
                        TokenKind::Word(word) => text.push_str(word),
                        _ => return None,
                    }
                }
                let factor = if text.is_empty() {
                    1.0
                } else {
                    text.parse::<f64>().ok()?
                };
                return Some(Length::TextWidth(factor));
            }
        }
    }
    parse_dimen_pt_at(&token_text(tokens), body).map(Length::Pt)
}

fn block_inlines(block: Block) -> Vec<Inline> {
    match block {
        Block::Paragraph(content)
        | Block::Heading { content, .. }
        | Block::FigureCaption { content }
        | Block::Styled { content, .. }
        | Block::ListItem { content, .. } => content,
        Block::VSpace { .. }
        | Block::Rule { .. }
        | Block::PageBreak
        | Block::Verbatim { .. }
        | Block::TableOfContents { .. }
        | Block::TitleBlock { .. }
        | Block::VFill => Vec::new(),
    }
}

/// The alignment preamble being built, mirroring `\@mkpream`'s state.
struct Preamble {
    columns: Vec<ColumnTemplate>,
    current: ColumnTemplate,
    placed: bool,
    first_amp: bool,
    fill: bool,
}

impl Preamble {
    fn new() -> Self {
        Preamble {
            columns: Vec::new(),
            current: empty_template(Align::Left),
            placed: false,
            first_amp: true,
            fill: false,
        }
    }

    fn add(&mut self, material: Material) {
        if self.placed {
            self.current.after.push(material);
        } else {
            self.current.before.push(material);
        }
    }

    /// `\@acol`.
    fn acol(&mut self) {
        self.add(Material::Space(TABCOLSEP_PT));
    }

    /// `\@addamp`.
    fn amp(&mut self) {
        if self.first_amp {
            self.first_amp = false;
        } else {
            self.current.fill_after = self.fill;
            let done = std::mem::replace(&mut self.current, empty_template(Align::Left));
            self.columns.push(done);
            self.placed = false;
        }
    }
}

fn empty_template(align: Align) -> ColumnTemplate {
    ColumnTemplate {
        before: Vec::new(),
        align,
        after: Vec::new(),
        fill_after: false,
    }
}

/// `\@mkpream`'s `\@lastchclass`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Last {
    Start,
    Column,
    Rule,
    At,
    AtArg,
    P,
    PArg,
}

impl P<'_> {
    fn body_pt(&self) -> f64 {
        self.class_size_pt.unwrap_or(crate::layout::BODY_SIZE_PT)
    }

    pub(super) fn tabular_environment(&mut self, open: Span, name: &str, para: &mut Vec<Inline>) {
        let space_before = self.t[..self.i]
            .iter()
            .rposition(|input| input.token.span == open)
            .is_none_or(|index| preceded_by_space(&self.t, index));
        let style = self.style;
        let body = self.body_pt();
        let width = if name == "tabular*" {
            let (tokens, argument_span) = self.required_group(name, open);
            let length = table_length(&tokens, body);
            if length.is_none() {
                self.diags.push(Diagnostic::error(
                    format!(
                        "tabular* width must be a dimension or a multiple of \\textwidth, got '{}'",
                        token_text(&tokens).trim()
                    ),
                    Some(argument_span),
                    Some("laid the table out at its natural width".into()),
                ));
            }
            length
        } else {
            None
        };
        let position = match self.optional_bracket_argument() {
            Some((raw, _)) if raw.trim() == "t" => VerticalPosition::Top,
            Some((raw, _)) if raw.trim() == "b" => VerticalPosition::Bottom,
            _ => VerticalPosition::Center,
        };
        let (spec_tokens, spec_span) = self.required_group(name, open);
        let arraystretch = self.array_stretch(open);
        let mut columns = self.column_templates(&spec_tokens, false);
        if columns.is_empty() {
            self.diags.push(Diagnostic::error(
                "tabular column specification has no columns",
                Some(spec_span),
                Some("laid the table out with one l column".into()),
            ));
            let mut template = empty_template(Align::Left);
            template.before.push(Material::Space(TABCOLSEP_PT));
            template.after.push(Material::Space(TABCOLSEP_PT));
            columns.push(template);
        }
        let n = columns.len();
        let booktabs = self.packages.iter().any(|package| package == "booktabs");

        let mut entries = Vec::new();
        let mut row = vec![RawCell::new(None)];
        let mut depth = 0usize;
        let mut end = spec_span.end;
        let mut found_end = false;
        while self.i < self.t.len() {
            if depth == 0 {
                if let Some((after, end_span)) = environment_end_at(&self.t, self.i, name) {
                    self.i = after;
                    end = end_span.end;
                    found_end = true;
                    break;
                }
            }
            let input = self.t[self.i].clone();
            let span = input.token.span;
            end = span.end;
            match &input.token.kind {
                TokenKind::Command(command) if depth == 0 && is_rule_command(command, booktabs) => {
                    self.i += 1;
                    let entry = self.tabular_rule(command, span, n, body);
                    if row_is_blank(&row) {
                        row = vec![RawCell::new(None)];
                        entries.extend(entry.map(RawEntry::Done));
                    } else {
                        self.diags.push(Diagnostic::error(
                            format!("\\{command} is only allowed at the start of a table row"),
                            Some(span),
                            Some("ignored the misplaced rule".into()),
                        ));
                    }
                    continue;
                }
                TokenKind::Command(command)
                    if depth == 0
                        && command == "multicolumn"
                        && row.last().is_some_and(RawCell::is_blank) =>
                {
                    self.i += 1;
                    let (count_tokens, count_span) = self.required_group(command, span);
                    let (spec, _) = self.required_group(command, span);
                    let (content, content_span) = self.required_group(command, span);
                    let raw = token_text(&count_tokens);
                    let count = match raw.trim().parse::<usize>() {
                        Ok(count) if count >= 1 => count,
                        _ => {
                            self.diags.push(Diagnostic::error(
                                format!(
                                    "\\multicolumn needs a positive column count, got '{}'",
                                    raw.trim()
                                ),
                                Some(count_span),
                                Some("spanned one column".into()),
                            ));
                            1
                        }
                    };
                    let cell = row.last_mut().expect("a row always has an entry");
                    cell.tokens = content;
                    cell.multicolumn = Some((count, spec, span.merge(content_span)));
                    continue;
                }
                TokenKind::Command(command) if depth == 0 && command == "tabularnewline" => {
                    self.i += 1;
                    let argument = self.row_end_argument(body);
                    let done = std::mem::replace(&mut row, vec![RawCell::new(None)]);
                    entries.push(RawEntry::Row(done, argument));
                    continue;
                }
                TokenKind::LineBreak if depth == 0 => {
                    self.i += 1;
                    let argument = self.row_end_argument(body);
                    let done = std::mem::replace(&mut row, vec![RawCell::new(None)]);
                    entries.push(RawEntry::Row(done, argument));
                    continue;
                }
                // `\&` lexes as a one-character word spanning two bytes.
                TokenKind::Word(word)
                    if depth == 0
                        && word.contains('&')
                        && !(word == "&"
                            && !input.maps_to_invocation
                            && span.end - span.start == 2) =>
                {
                    let exact = span.end - span.start == word.len();
                    for (index, piece) in word.split('&').enumerate() {
                        if index > 0 {
                            row.push(RawCell::new(Some(span)));
                        }
                        if piece.is_empty() {
                            continue;
                        }
                        let offset = piece.as_ptr() as usize - word.as_ptr() as usize;
                        let piece_span = if exact {
                            Span::in_document(
                                span.document,
                                span.start + offset,
                                span.start + offset + piece.len(),
                            )
                        } else {
                            span
                        };
                        row.last_mut()
                            .expect("a row always has an entry")
                            .tokens
                            .push(InputToken {
                                token: Token {
                                    kind: TokenKind::Word(piece.to_string()),
                                    span: piece_span,
                                },
                                definition: input.definition,
                                maps_to_invocation: input.maps_to_invocation,
                            });
                    }
                }
                TokenKind::RBrace if depth == 0 => {
                    self.diags.push(Diagnostic::error(
                        "unmatched '}' inside a table row",
                        Some(span),
                        Some("ignored the stray brace and continued".into()),
                    ));
                }
                kind => {
                    match kind {
                        TokenKind::LBrace => depth += 1,
                        TokenKind::Command(command) if command == "begin" => depth += 1,
                        TokenKind::RBrace => depth = depth.saturating_sub(1),
                        TokenKind::Command(command) if command == "end" => {
                            depth = depth.saturating_sub(1)
                        }
                        _ => {}
                    }
                    row.last_mut()
                        .expect("a row always has an entry")
                        .tokens
                        .push(input);
                }
            }
            self.i += 1;
        }
        if !found_end {
            self.diags.push(Diagnostic::error(
                format!("unterminated environment '{name}' — no matching \\end"),
                Some(open),
                Some("closed the table at end of input".into()),
            ));
        }
        if !row_is_blank(&row) {
            entries.push(RawEntry::Row(row, None));
        }

        let mut out = Vec::new();
        for entry in entries {
            let (cells, argument) = match entry {
                RawEntry::Done(entry) => {
                    out.push(entry);
                    continue;
                }
                RawEntry::Row(cells, argument) => (cells, argument),
            };
            let mut row_cells = Vec::new();
            let mut column = 0;
            for raw in cells {
                if column >= n {
                    // TeX changes the extra `&` into `\cr`.
                    self.diags.push(Diagnostic::error(
                        format!("extra alignment tab: this row has more entries than the {n} columns of the table"),
                        Some(raw.amp.unwrap_or(open)),
                        Some("started a new row at the extra entry, as TeX does".into()),
                    ));
                    out.push(Entry::Row(Row {
                        cells: std::mem::take(&mut row_cells),
                        extra_depth_pt: 0.0,
                    }));
                    column = 0;
                }
                let (mut columns_spanned, template) = match raw.multicolumn {
                    Some((count, spec, multicolumn_span)) => {
                        let mut templates = self.column_templates(&spec, true);
                        if templates.len() != 1 {
                            self.diags.push(Diagnostic::error(
                                "\\multicolumn specification must describe exactly one column",
                                Some(multicolumn_span),
                                Some(if templates.is_empty() {
                                    "centred the entry without rules".into()
                                } else {
                                    "used the first column of the specification".into()
                                }),
                            ));
                        }
                        let template = if templates.is_empty() {
                            let mut template = empty_template(Align::Center);
                            template.before.push(Material::Space(TABCOLSEP_PT));
                            template.after.push(Material::Space(TABCOLSEP_PT));
                            template
                        } else {
                            templates.swap_remove(0)
                        };
                        if column + count > n {
                            self.diags.push(Diagnostic::error(
                                format!("\\multicolumn{{{count}}} spans past the last of the {n} columns"),
                                Some(multicolumn_span),
                                Some("spanned to the last column".into()),
                            ));
                        }
                        (count, Some(template))
                    }
                    None => (1, None),
                };
                columns_spanned = columns_spanned.min(n - column);
                let content = self.tabular_cell_inlines(raw.tokens);
                row_cells.push(Cell {
                    content,
                    columns: columns_spanned,
                    template,
                });
                column += columns_spanned;
            }
            out.push(Entry::Row(Row {
                cells: row_cells,
                extra_depth_pt: argument.filter(|pt| *pt > 0.0).unwrap_or(0.0),
            }));
            if let Some(pt) = argument.filter(|pt| *pt <= 0.0) {
                out.push(Entry::VSpace { pt });
            }
        }

        para.push(Inline::Tabular(Box::new(Tabular {
            columns,
            entries: out,
            position,
            width,
            arraystretch,
            style,
            span: Span::in_document(open.document, open.start, end),
            space_before,
        })));
    }

    /// `\arraystretch` as currently defined (`1` by default).
    fn array_stretch(&mut self, open: Span) -> f64 {
        // The expansion pass records the replacement text in effect at this
        // `\begin`; a missing entry means `\arraystretch` was undefined or
        // given parameters, which is not a plain number either.
        let Some(text) = self.arraystretch.get(&(open.document.0, open.start)).cloned() else {
            return 1.0;
        };
        match text.trim().parse::<f64>() {
            Ok(value) if value.is_finite() && value >= 0.0 => {
                value
            }
            _ => {
                self.diags.push(Diagnostic::warning(
                    format!(
                        "\\arraystretch must be a plain number, got '{}'",
                        text.trim()
                    ),
                    Some(open),
                    Some("used an \\arraystretch of 1".into()),
                ));
                1.0
            }
        }
    }

    /// The optional `*` and `[<dimen>]` after a row's `\\`.
    fn row_end_argument(&mut self, body: f64) -> Option<f64> {
        self.skip_spaces();
        if let Some(input) = self.t.get_mut(self.i) {
            if let TokenKind::Word(word) = &input.token.kind {
                if let Some(rest) = word.strip_prefix('*') {
                    if rest.is_empty() {
                        self.i += 1;
                    } else {
                        let span = input.token.span;
                        if span.end - span.start == word.len() {
                            input.token.span =
                                Span::in_document(span.document, span.start + 1, span.end);
                        }
                        input.token.kind = TokenKind::Word(rest.to_string());
                    }
                    self.skip_spaces();
                }
            }
        }
        // `\\[2pt]Next`: keep the text glued after `]` as the next entry's.
        let glued = match self.t.get(self.i).map(|input| &input.token) {
            Some(Token {
                kind: TokenKind::Word(word),
                span,
            }) if word.starts_with('[') && word.contains(']') => {
                let close = word.find(']').expect("checked");
                Some((
                    word[1..close].to_string(),
                    word[close + 1..].to_string(),
                    *span,
                    word.len(),
                ))
            }
            _ => None,
        };
        let (raw, span) = match glued {
            Some((raw, rest, span, len)) => {
                if rest.is_empty() {
                    self.i += 1;
                } else {
                    let input = &mut self.t[self.i];
                    if span.end - span.start == len {
                        input.token.span = Span::in_document(
                            span.document,
                            span.start + len - rest.len(),
                            span.end,
                        );
                    }
                    input.token.kind = TokenKind::Word(rest);
                }
                (raw, span)
            }
            None => self.optional_bracket_argument()?,
        };
        match parse_dimen_pt_at(&raw, body) {
            Some(pt) => Some(pt),
            None => {
                self.diags.push(Diagnostic::error(
                    format!(
                        "\\\\[...] in a table requires a recognised dimension, got '{}'",
                        raw.trim()
                    ),
                    Some(span),
                    Some("ignored the extra row space".into()),
                ));
                None
            }
        }
    }

    /// A rule at the start of a row; `None` after a diagnosed malformed one.
    fn tabular_rule(&mut self, command: &str, span: Span, n: usize, body: f64) -> Option<Entry> {
        match command {
            "hline" => Some(Entry::HLine { span }),
            "cline" => {
                let (tokens, argument_span) = self.required_group(command, span);
                let span = span.merge(argument_span);
                let (first, last) = self.column_range(command, &token_text(&tokens), n, span)?;
                Some(Entry::CLine { first, last, span })
            }
            "toprule" | "midrule" | "bottomrule" => {
                let (width_pt, span) = self.rule_width(command, span, body);
                let kind = match command {
                    "toprule" => BookRule::Top,
                    "midrule" => BookRule::Mid,
                    _ => BookRule::Bottom,
                };
                Some(Entry::BookRule {
                    kind,
                    width_pt,
                    span,
                })
            }
            _ => {
                let (width_pt, span) = self.rule_width(command, span, body);
                let (trim_left, trim_right) = self.cmidrule_trim();
                let (tokens, argument_span) = self.required_group(command, span);
                let span = span.merge(argument_span);
                let (first, last) = self.column_range(command, &token_text(&tokens), n, span)?;
                Some(Entry::CMidRule {
                    first,
                    last,
                    trim_left,
                    trim_right,
                    width_pt,
                    span,
                })
            }
        }
    }

    fn rule_width(&mut self, command: &str, span: Span, body: f64) -> (Option<f64>, Span) {
        let Some((raw, option_span)) = self.optional_bracket_argument() else {
            return (None, span);
        };
        let span = span.merge(option_span);
        match parse_dimen_pt_at(&raw, body) {
            Some(pt) if pt > 0.0 => (Some(pt), span),
            _ => {
                self.diags.push(Diagnostic::error(
                    format!(
                        "\\{command} rule width must be a positive dimension, got '{}'",
                        raw.trim()
                    ),
                    Some(option_span),
                    Some("used the default rule width".into()),
                ));
                (None, span)
            }
        }
    }

    /// booktabs `\cmidrule(lr)` trimming.
    fn cmidrule_trim(&mut self) -> (bool, bool) {
        self.skip_spaces();
        let Some(TokenKind::Word(word)) = self.peek().map(|token| token.kind.clone()) else {
            return (false, false);
        };
        let Some(inner) = word.strip_prefix('(').and_then(|rest| rest.split_once(')')) else {
            return (false, false);
        };
        let (trim, rest) = (inner.0.to_string(), inner.1.to_string());
        if rest.is_empty() {
            self.i += 1;
        } else if let Some(input) = self.t.get_mut(self.i) {
            let span = input.token.span;
            if span.end - span.start == word.len() {
                let consumed = word.len() - rest.len();
                input.token.span =
                    Span::in_document(span.document, span.start + consumed, span.end);
            }
            input.token.kind = TokenKind::Word(rest);
        }
        (trim.contains('l'), trim.contains('r'))
    }

    fn column_range(
        &mut self,
        command: &str,
        raw: &str,
        n: usize,
        span: Span,
    ) -> Option<(usize, usize)> {
        let range = raw.trim().split_once('-').and_then(|(first, last)| {
            Some((
                first.trim().parse::<usize>().ok()?,
                last.trim().parse::<usize>().ok()?,
            ))
        });
        match range {
            Some((first, last)) if 1 <= first && first <= last && last <= n => {
                Some((first - 1, last - 1))
            }
            _ => {
                self.diags.push(Diagnostic::error(
                    format!(
                        "\\{command}{{{}}} must name a column range within columns 1-{n}",
                        raw.trim()
                    ),
                    Some(span),
                    Some("omitted the rule".into()),
                ));
                None
            }
        }
    }

    /// Parses one table entry (or `@{}` text) as its own group.
    fn tabular_cell_inlines(&mut self, tokens: Vec<InputToken>) -> Vec<Inline> {
        let first_span = tokens
            .iter()
            .find(|input| !matches!(input.token.kind, TokenKind::Space | TokenKind::Comment))
            .map(|input| input.token.span);
        // A blank line inside an entry is ignored: `\@array` redefines `\par`.
        let tokens = tokens
            .into_iter()
            .map(|mut input| {
                if input.token.kind == TokenKind::ParBreak {
                    input.token.kind = TokenKind::Space;
                }
                input
            })
            .collect();
        let outer_tokens = std::mem::replace(&mut self.t, tokens);
        let outer_index = std::mem::replace(&mut self.i, 0);
        let style = self.style;
        let style_depth = self.style_stack.len();
        let brace_depth = self.brace_stack.len();
        let dependency_count = self.block_dependencies.len();

        let mut blocks = Vec::new();
        let mut para = Vec::new();
        self.parse_stream(&mut blocks, &mut para);

        while self.brace_stack.len() > brace_depth {
            let open = self.brace_stack.pop().expect("length checked");
            self.diags.push(Diagnostic::error(
                "unmatched '{' — group never closed inside a table entry",
                Some(open),
                Some("closed the group at the end of the entry".into()),
            ));
        }
        self.style = style;
        self.style_stack.truncate(style_depth);
        self.t = outer_tokens;
        self.i = outer_index;

        if blocks.is_empty() {
            return para;
        }
        self.diags.push(Diagnostic::warning(
            "block-level content (paragraphs, headings, lists, displays) is not supported inside a table entry",
            first_span,
            Some("kept its text inline in the entry".into()),
        ));
        // The entry belongs to the enclosing paragraph block, so fold the
        // macro dependencies of the blocks it produced back into that block.
        for dependency in self.block_dependencies.drain(dependency_count..).flatten() {
            self.current_dependencies.insert(
                dependency.name,
                (dependency.argument_count, dependency.replacement),
            );
        }
        let mut content: Vec<Inline> = blocks.into_iter().flat_map(block_inlines).collect();
        content.extend(para);
        content
    }

    /// Builds the alignment preamble exactly as `\@mkpream` does.
    fn column_templates(
        &mut self,
        tokens: &[InputToken],
        multicolumn: bool,
    ) -> Vec<ColumnTemplate> {
        let body = self.body_pt();
        let mut items = Vec::new();
        self.spec_items(tokens, &mut items, 0);
        let mut pre = Preamble::new();
        let mut last = Last::Start;
        let mut skip_group = false;
        for item in items {
            if skip_group {
                skip_group = false;
                if matches!(item, SpecItem::Group(..)) {
                    continue;
                }
            }
            match (last, item) {
                (Last::At, SpecItem::Group(group, _)) => {
                    self.at_expression(group, &mut pre, multicolumn);
                    last = Last::AtArg;
                }
                (Last::P, SpecItem::Group(group, span)) => {
                    let width = table_length(&group, body).unwrap_or_else(|| {
                        self.diags.push(Diagnostic::error(
                            format!(
                                "p column width must be a dimension or a multiple of \\textwidth, got '{}'",
                                token_text(&group).trim()
                            ),
                            Some(span),
                            Some("used a zero width".into()),
                        ));
                        Length::Pt(0.0)
                    });
                    pre.current.align = Align::Paragraph(width);
                    pre.placed = true;
                    last = Last::PArg;
                }
                (Last::At | Last::P, item) => {
                    let span = match item {
                        SpecItem::Char(_, span)
                        | SpecItem::Group(_, span)
                        | SpecItem::Command(_, span) => span,
                    };
                    self.diags.push(Diagnostic::error(
                        if last == Last::At {
                            "@ in a column specification needs a braced expression"
                        } else {
                            "p column needs a braced width"
                        },
                        Some(span),
                        Some("ignored the malformed column".into()),
                    ));
                    if last == Last::P {
                        pre.current.align = Align::Left;
                        pre.placed = true;
                        last = Last::Column;
                    } else {
                        last = Last::AtArg;
                    }
                }
                (_, SpecItem::Char('|', span)) => {
                    match last {
                        Last::Column | Last::PArg => {
                            pre.acol();
                            pre.add(Material::Rule(span));
                        }
                        Last::Rule => {
                            pre.add(Material::Space(DOUBLERULESEP_PT));
                            pre.add(Material::Rule(span));
                        }
                        _ => pre.add(Material::Rule(span)),
                    }
                    last = Last::Rule;
                }
                (_, SpecItem::Char('@', _)) => {
                    if last == Last::Rule {
                        pre.add(Material::Space(ARRAYRULEWIDTH_PT / 2.0));
                    }
                    last = Last::At;
                }
                (_, SpecItem::Char(ch @ ('p' | 'm' | 'b'), span)) => {
                    if ch != 'p' {
                        self.diags.push(Diagnostic::warning(
                            format!(
                                "'{ch}' columns need the array package, which is not implemented"
                            ),
                            Some(span),
                            Some("laid the column out as a top-aligned p column".into()),
                        ));
                    }
                    start_column(&mut pre, last);
                    last = Last::P;
                }
                (_, SpecItem::Char(ch @ ('>' | '<' | '!'), span)) => {
                    self.diags.push(Diagnostic::warning(
                        format!("'{ch}{{...}}' in a column specification needs the array package, which is not implemented"),
                        Some(span),
                        Some("ignored the declaration".into()),
                    ));
                    skip_group = true;
                }
                (_, SpecItem::Char(ch, span)) => {
                    let align = match ch {
                        'l' => Align::Left,
                        'c' => Align::Center,
                        'r' => Align::Right,
                        _ => {
                            self.diags.push(Diagnostic::error(
                                format!("illegal character '{ch}' in the column specification"),
                                Some(span),
                                Some("laid the column out as l".into()),
                            ));
                            Align::Left
                        }
                    };
                    start_column(&mut pre, last);
                    pre.current.align = align;
                    pre.placed = true;
                    last = Last::Column;
                }
                (_, SpecItem::Group(_, span)) => {
                    self.diags.push(Diagnostic::error(
                        "unexpected braced group in the column specification",
                        Some(span),
                        Some("ignored the group".into()),
                    ));
                }
                (_, SpecItem::Command(name, span)) => {
                    self.diags.push(Diagnostic::error(
                        format!("\\{name} is not supported in a column specification"),
                        Some(span),
                        Some("ignored the command".into()),
                    ));
                }
            }
        }
        match last {
            Last::Column | Last::PArg => pre.acol(),
            Last::At | Last::P => self.diags.push(Diagnostic::error(
                "column specification ends before its last @ or p argument",
                tokens.last().map(|input| input.token.span),
                Some("ignored the incomplete column".into()),
            )),
            _ => {}
        }
        if !pre.first_amp {
            // `\tabskip\z@skip` precedes the preamble's `\cr`.
            pre.current.fill_after = false;
            pre.columns.push(pre.current);
        }
        pre.columns
    }

    /// `@{...}`: `\extracolsep` sets the `\tabskip` glue; anything else is
    /// text material in the template.
    fn at_expression(&mut self, tokens: Vec<InputToken>, pre: &mut Preamble, multicolumn: bool) {
        let mut rest = Vec::new();
        let mut index = 0;
        while index < tokens.len() {
            let input = &tokens[index];
            if !matches!(&input.token.kind, TokenKind::Command(name) if name == "extracolsep") {
                rest.push(input.clone());
                index += 1;
                continue;
            }
            let mut cursor = index + 1;
            while matches!(
                tokens.get(cursor).map(|input| &input.token.kind),
                Some(TokenKind::Space | TokenKind::Comment)
            ) {
                cursor += 1;
            }
            let mut group_end = cursor;
            if matches!(
                tokens.get(cursor).map(|input| &input.token.kind),
                Some(TokenKind::LBrace)
            ) {
                let mut depth = 0usize;
                for (offset, input) in tokens[cursor..].iter().enumerate() {
                    match input.token.kind {
                        TokenKind::LBrace => depth += 1,
                        TokenKind::RBrace => {
                            depth -= 1;
                            if depth == 0 {
                                group_end = cursor + offset;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }
            let value = if group_end > cursor {
                token_text(&tokens[cursor + 1..group_end])
            } else {
                String::new()
            };
            let value = value.trim();
            if value == "fill" {
                // Inside `\multicolumn` the assignment is local to the entry.
                if !multicolumn {
                    pre.fill = true;
                }
            } else if parse_dimen_pt_at(value, self.body_pt()) == Some(0.0) {
                if !multicolumn {
                    pre.fill = false;
                }
            } else {
                self.diags.push(Diagnostic::warning(
                    format!("\\extracolsep{{{value}}} is not implemented; only \\fill and 0pt are"),
                    Some(input.token.span),
                    Some("ignored the inter-column space".into()),
                ));
            }
            index = group_end.max(index) + 1;
        }
        if !blank(&rest) {
            let content = self.tabular_cell_inlines(rest);
            pre.add(Material::Text(content));
        }
    }

    /// Flattens the specification into characters and groups, expanding
    /// `*{n}{...}` as `\@expast` does.
    fn spec_items(&mut self, tokens: &[InputToken], out: &mut Vec<SpecItem>, depth: usize) {
        let mut raw = Vec::new();
        let mut index = 0;
        while index < tokens.len() {
            let input = &tokens[index];
            let span = input.token.span;
            match &input.token.kind {
                TokenKind::Word(word) => {
                    let exact = span.end - span.start == word.len();
                    for (offset, ch) in word.char_indices() {
                        let ch_span = if exact {
                            Span::in_document(
                                span.document,
                                span.start + offset,
                                span.start + offset + ch.len_utf8(),
                            )
                        } else {
                            span
                        };
                        raw.push(SpecItem::Char(ch, ch_span));
                    }
                }
                TokenKind::LBrace => {
                    let mut group_depth = 0usize;
                    let mut close = tokens.len();
                    for (offset, input) in tokens[index..].iter().enumerate() {
                        match input.token.kind {
                            TokenKind::LBrace => group_depth += 1,
                            TokenKind::RBrace => {
                                group_depth -= 1;
                                if group_depth == 0 {
                                    close = index + offset;
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                    let end = tokens.get(close).map_or(span, |input| input.token.span);
                    raw.push(SpecItem::Group(
                        tokens[index + 1..close].to_vec(),
                        span.merge(end),
                    ));
                    index = close;
                }
                TokenKind::Command(name) => raw.push(SpecItem::Command(name.clone(), span)),
                _ => {}
            }
            index += 1;
        }
        let mut items = raw.into_iter().peekable();
        while let Some(item) = items.next() {
            if out.len() >= MAX_SPEC_ITEMS {
                return;
            }
            let SpecItem::Char('*', star_span) = item else {
                out.push(item);
                continue;
            };
            let count = match items.next() {
                Some(SpecItem::Group(count, _)) => token_text(&count).trim().parse::<usize>().ok(),
                _ => None,
            };
            let repeated = match items.next() {
                Some(SpecItem::Group(repeated, _)) => Some(repeated),
                _ => None,
            };
            match (count, repeated) {
                (Some(count), Some(repeated)) if depth < MAX_SPEC_DEPTH => {
                    for _ in 0..count {
                        if out.len() >= MAX_SPEC_ITEMS {
                            break;
                        }
                        self.spec_items(&repeated, out, depth + 1);
                    }
                }
                _ => self.diags.push(Diagnostic::error(
                    "*{n}{columns} in a column specification needs a count and a braced group",
                    Some(star_span),
                    Some("ignored the repetition".into()),
                )),
            }
        }
    }
}

/// `\@classz`/`\@classiii`: the glue and `&` before a new column's entry.
fn start_column(pre: &mut Preamble, last: Last) {
    match last {
        Last::Column | Last::PArg => {
            pre.acol();
            pre.amp();
            pre.acol();
        }
        Last::AtArg => pre.amp(),
        Last::Start | Last::Rule | Last::At | Last::P => {
            pre.amp();
            pre.acol();
        }
    }
}
