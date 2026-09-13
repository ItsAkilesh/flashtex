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
    Align, BookRule, BoxAlign, Cell, ColumnTemplate, Entry, Length, Material, Row, Tabular,
    VerticalPosition, ARRAYRULEWIDTH_PT, DOUBLERULESEP_PT, TABCOLSEP_PT,
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

/// Packages that load array.sty (and so replace the kernel's `\@mkpream`).
const ARRAY_PACKAGES: &[&str] = &[
    "array", "tabularx", "tabulary", "dcolumn", "delarray", "colortbl", "arydshln",
];

/// One column's array-package declarations: the tokens `>{}` inserts before
/// every entry and `<{}` after it (array.sty `\insert@column`).
#[derive(Debug, Clone, Default)]
struct Decls {
    before: Vec<InputToken>,
    after: Vec<InputToken>,
}

/// The alignment preamble being built, mirroring `\@mkpream`'s state.
struct Preamble {
    columns: Vec<ColumnTemplate>,
    current: ColumnTemplate,
    decls: Vec<Decls>,
    current_decls: Decls,
    /// `>{}` tokens waiting for the next column (array.sty `\toks\count@`).
    pending_before: Vec<InputToken>,
    placed: bool,
    first_amp: bool,
    fill: bool,
}

/// Prepends a declaration as array.sty's `\save@decl` does
/// (`\toks\count@ = {\@nextchar\the\toks\count@}`).
fn prepend(target: &mut Vec<InputToken>, tokens: Vec<InputToken>) {
    let old = std::mem::replace(target, tokens);
    target.extend(old);
}

impl Preamble {
    fn new() -> Self {
        Preamble {
            columns: Vec::new(),
            current: empty_template(Align::Left),
            decls: Vec::new(),
            current_decls: Decls::default(),
            pending_before: Vec::new(),
            placed: false,
            first_amp: true,
            fill: false,
        }
    }

    fn finish(mut self) -> (Vec<ColumnTemplate>, Vec<Decls>) {
        if !self.first_amp {
            // `\tabskip\z@skip` precedes the preamble's `\cr`.
            self.current.fill_after = false;
            self.columns.push(self.current);
            self.decls.push(self.current_decls);
        }
        (self.columns, self.decls)
    }

    /// array.sty `\@classvi`: what precedes a `|` or `!{}`.
    fn array_classvi(&mut self, last: u8) {
        match last {
            0 | 2 => self.acol(),
            1 => self.add(Material::Space(DOUBLERULESEP_PT)),
            _ => {}
        }
    }

    /// array.sty `\@classx`: what precedes a new column (or its `>{}`).
    fn array_classx(&mut self, last: u8) {
        match last {
            0 | 2 => {
                self.acol();
                self.amp();
                self.acol();
            }
            1 => {
                self.amp();
                self.acol();
            }
            4 => {
                self.amp();
                self.acol();
            }
            5 => self.amp(),
            _ => {}
        }
    }

    /// array.sty `\@classz`: the column entry itself.
    fn array_classz(&mut self, last: u8, align: Align) {
        self.array_classx(last);
        self.current.align = align;
        self.placed = true;
        self.current_decls.before = std::mem::take(&mut self.pending_before);
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
            self.decls.push(std::mem::take(&mut self.current_decls));
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
        let array_package = self.array_package();
        let (mut columns, mut decls) = self.column_templates(&spec_tokens, false);
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
            decls = vec![Decls::default()];
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
                let (mut columns_spanned, template, cell_decls) = match raw.multicolumn {
                    Some((count, spec, multicolumn_span)) => {
                        let (mut templates, mut own_decls) = self.column_templates(&spec, true);
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
                        let (template, own) = if templates.is_empty() {
                            let mut template = empty_template(Align::Center);
                            template.before.push(Material::Space(TABCOLSEP_PT));
                            template.after.push(Material::Space(TABCOLSEP_PT));
                            (template, Decls::default())
                        } else {
                            (templates.swap_remove(0), own_decls.swap_remove(0))
                        };
                        if column + count > n {
                            self.diags.push(Diagnostic::error(
                                format!("\\multicolumn{{{count}}} spans past the last of the {n} columns"),
                                Some(multicolumn_span),
                                Some("spanned to the last column".into()),
                            ));
                        }
                        (count, Some(template), own)
                    }
                    None => (1, None, decls.get(column).cloned().unwrap_or_default()),
                };
                columns_spanned = columns_spanned.min(n - column);
                let align = template.as_ref().unwrap_or(&columns[column]).align;
                // array.sty `\insert@column`: the `>{}` tokens, the entry,
                // then the `<{}` tokens, all in the entry's one group.
                let declarations = !cell_decls.before.is_empty() || !cell_decls.after.is_empty();
                let mut tokens = cell_decls.before;
                tokens.extend(raw.tokens);
                tokens.extend(cell_decls.after);
                let outer_alignment = self.declared_alignment.take();
                let content = self.tabular_cell_inlines(tokens);
                let alignment =
                    std::mem::replace(&mut self.declared_alignment, outer_alignment);
                row_cells.push(Cell {
                    content,
                    columns: columns_spanned,
                    template,
                    alignment: align.paragraph_width().and(alignment),
                    declarations,
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
            array_package,
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
        if let Some(input) = self.token_mut(self.i) {
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
                    let index = self.i;
                    let input = self.token_mut(index).expect("glued word is at the cursor");
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
        } else if let Some(input) = self.token_mut(self.i) {
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
        let outer_tokens = std::mem::replace(&mut self.t, std::rc::Rc::new(tokens));
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
        let mut content: Vec<Inline> = blocks.into_iter().flat_map(block_inlines).collect::<Vec<_>>().into();
        content.extend(para);
        content
    }

    fn array_package(&self) -> bool {
        self.packages
            .iter()
            .any(|package| ARRAY_PACKAGES.contains(&package.as_str()))
    }

    /// Builds the alignment preamble exactly as `\@mkpream` does (the
    /// kernel's, or array.sty's when that package is loaded).
    fn column_templates(
        &mut self,
        tokens: &[InputToken],
        multicolumn: bool,
    ) -> (Vec<ColumnTemplate>, Vec<Decls>) {
        if self.array_package() {
            return self.array_column_templates(tokens, multicolumn);
        }
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
        pre.finish()
    }

    /// array.sty v2.6n's `\@mkpream` (array.sty 385-416, classes from
    /// `\@testpach` 56-81): `\@lastchclass` is 0 after a column, 1 after `|`
    /// or a `!{}` argument, 2 after a `<{}` argument, 3 after a `>{}`
    /// argument, 4 at the start, 5 after an `@{}` argument, 6/7/8/9 after a
    /// bare `!`/`@`/`<`/`>`, 10 after `m`/`p`/`b` (and here `w`/`W`).
    fn array_column_templates(
        &mut self,
        tokens: &[InputToken],
        multicolumn: bool,
    ) -> (Vec<ColumnTemplate>, Vec<Decls>) {
        #[derive(Clone, Copy)]
        enum Pending {
            Par(char),
            FixedAlign,
            FixedWidth(BoxAlign),
        }
        let body = self.body_pt();
        let mut items = Vec::new();
        self.spec_items(tokens, &mut items, 0);
        let mut pre = Preamble::new();
        let mut last = 4u8;
        let mut pending = Pending::Par('p');
        for item in items {
            if matches!(last, 6..=10) {
                let SpecItem::Group(group, span) = item else {
                    let span = match item {
                        SpecItem::Char(_, span)
                        | SpecItem::Group(_, span)
                        | SpecItem::Command(_, span) => span,
                    };
                    self.diags.push(Diagnostic::error(
                        "array column specification: missing braced argument",
                        Some(span),
                        Some("ignored the token's argument".into()),
                    ));
                    last = match last {
                        6 => 1,
                        7 => 5,
                        8 => 2,
                        9 => 3,
                        _ => {
                            pre.array_classz(last, Align::Left);
                            0
                        }
                    };
                    continue;
                };
                last = match (last, pending) {
                    // `!{}`: `\@classi` with `\@chnum` 1, i.e. `\@classv`.
                    (6, _) => {
                        self.at_expression(group, &mut pre, multicolumn);
                        1
                    }
                    (7, _) => {
                        self.at_expression(group, &mut pre, multicolumn);
                        5
                    }
                    // `\@classii`: prepended to this column's `<` tokens.
                    (8, _) => {
                        prepend(&mut pre.current_decls.after, group);
                        2
                    }
                    (9, _) => {
                        prepend(&mut pre.pending_before, group);
                        3
                    }
                    (_, Pending::Par(ch)) => {
                        let width = self.column_width(&group, span, body);
                        let align = match ch {
                            'm' => Align::Middle(width),
                            'b' => Align::Bottom(width),
                            _ => Align::Paragraph(width),
                        };
                        pre.array_classz(10, align);
                        0
                    }
                    (_, Pending::FixedAlign) => {
                        let align = match token_text(&group).trim() {
                            "c" => BoxAlign::Center,
                            "r" => BoxAlign::Right,
                            "l" | "s" => BoxAlign::Left,
                            other => {
                                self.diags.push(Diagnostic::error(
                                    format!("w/W column alignment must be l, c, r or s, got '{other}'"),
                                    Some(span),
                                    Some("aligned the entry left".into()),
                                ));
                                BoxAlign::Left
                            }
                        };
                        pending = Pending::FixedWidth(align);
                        10
                    }
                    (_, Pending::FixedWidth(align)) => {
                        let width = self.column_width(&group, span, body);
                        pre.array_classz(10, Align::Fixed(width, align));
                        0
                    }
                };
                continue;
            }
            match item {
                SpecItem::Char(ch @ ('l' | 'c' | 'r'), _) => {
                    let align = match ch {
                        'l' => Align::Left,
                        'c' => Align::Center,
                        _ => Align::Right,
                    };
                    pre.array_classz(last, align);
                    last = 0;
                }
                SpecItem::Char('|', span) => {
                    pre.array_classvi(last);
                    // `\@arrayrule`: `\vline`, `\vrule\@width\arrayrulewidth`.
                    pre.add(Material::VLine {
                        span,
                        width_pt: None,
                    });
                    last = 1;
                }
                SpecItem::Char('!', _) => {
                    pre.array_classvi(last);
                    last = 6;
                }
                SpecItem::Char('@', span) => {
                    if last == 3 {
                        self.diags.push(Diagnostic::error(
                            ">{..} at wrong position: token ignored",
                            Some(span),
                            None,
                        ));
                    }
                    last = 7;
                }
                SpecItem::Char('<', span) => {
                    // `\@classviii`: a `<` not after a column is a `!`.
                    if last > 0 && last != 2 {
                        self.diags.push(Diagnostic::error(
                            "<{..} at wrong position: changed to !{..}",
                            Some(span),
                            None,
                        ));
                        pre.array_classvi(last);
                        last = 6;
                    } else {
                        last = 8;
                    }
                }
                SpecItem::Char('>', _) => {
                    pre.array_classx(last);
                    last = 9;
                }
                SpecItem::Char(ch @ ('m' | 'p' | 'b' | 'w' | 'W'), _) => {
                    pre.array_classx(last);
                    pending = if matches!(ch, 'w' | 'W') {
                        Pending::FixedAlign
                    } else {
                        Pending::Par(ch)
                    };
                    last = 10;
                }
                SpecItem::Char(ch, span) => {
                    self.diags.push(Diagnostic::error(
                        format!("illegal character '{ch}' in the column specification"),
                        Some(span),
                        Some("laid the column out as c".into()),
                    ));
                    pre.array_classz(last, Align::Center);
                    last = 0;
                }
                SpecItem::Group(_, span) => {
                    self.diags.push(Diagnostic::error(
                        "unexpected braced group in the column specification",
                        Some(span),
                        Some("ignored the group".into()),
                    ));
                }
                SpecItem::Command(name, span) => {
                    self.diags.push(Diagnostic::error(
                        format!("\\{name} is not supported in a column specification"),
                        Some(span),
                        Some("ignored the command".into()),
                    ));
                }
            }
        }
        match last {
            0 | 2 => pre.acol(),
            1 | 4 | 5 => {}
            _ => self.diags.push(Diagnostic::error(
                "column specification ends before the argument of its last token",
                tokens.last().map(|input| input.token.span),
                Some("ignored the incomplete column".into()),
            )),
        }
        pre.finish()
    }

    fn column_width(&mut self, group: &[InputToken], span: Span, body: f64) -> Length {
        table_length(group, body).unwrap_or_else(|| {
            self.diags.push(Diagnostic::error(
                format!(
                    "column width must be a dimension or a multiple of \\textwidth, got '{}'",
                    token_text(group).trim()
                ),
                Some(span),
                Some("used a zero width".into()),
            ));
            Length::Pt(0.0)
        })
    }

    /// `\vline` (`\vrule\@width\arrayrulewidth`, latex.ltx 16736) or
    /// `\vrule` with at most a `width`, as the whole of an `@{}`/`!{}`
    /// expression: a rule running the row that takes its width.
    fn rule_material(&self, tokens: &[InputToken]) -> Option<Material> {
        let significant: Vec<&InputToken> = tokens
            .iter()
            .filter(|input| !matches!(input.token.kind, TokenKind::Space | TokenKind::Comment))
            .collect();
        let (first, rest) = significant.split_first()?;
        let TokenKind::Command(name) = &first.token.kind else {
            return None;
        };
        let span = first.token.span;
        match (name.as_str(), rest) {
            ("vline", []) => Some(Material::VLine {
                span,
                width_pt: None,
            }),
            // TeX's default rule width is 0.4pt (§463).
            ("vrule", []) => Some(Material::VLine {
                span,
                width_pt: Some(0.4),
            }),
            ("vrule", [keyword, dimen @ ..]) => {
                let TokenKind::Word(word) = &keyword.token.kind else {
                    return None;
                };
                let text: String = if word == "width" {
                    dimen
                        .iter()
                        .map(|input| match &input.token.kind {
                            TokenKind::Word(w) => Some(w.as_str()),
                            _ => None,
                        })
                        .collect::<Option<String>>()?
                } else {
                    word.strip_prefix("width")?.to_string()
                        + &dimen
                            .iter()
                            .map(|input| match &input.token.kind {
                                TokenKind::Word(w) => Some(w.as_str()),
                                _ => None,
                            })
                            .collect::<Option<String>>()?
                };
                let pt = parse_dimen_pt_at(&text, self.body_pt())?;
                Some(Material::VLine {
                    span,
                    width_pt: Some(pt),
                })
            }
            _ => None,
        }
    }

    /// array.sty 336-347: `\newcolumntype{X}[n]{spec}`. The definition is
    /// expanded wherever `X` appears in a later column specification.
    pub(super) fn new_column_type(&mut self, span: Span) {
        let (name_tokens, name_span) = self.required_group("newcolumntype", span);
        let name = token_text(&name_tokens);
        let mut chars = name.trim().chars();
        let (Some(ch), None) = (chars.next(), chars.next()) else {
            self.diags.push(Diagnostic::error(
                format!(
                    "\\newcolumntype needs a single character, got '{}'",
                    name.trim()
                ),
                Some(name_span),
                Some("ignored the definition".into()),
            ));
            let _ = self.optional_bracket_argument();
            let _ = self.required_group("newcolumntype", span);
            return;
        };
        let count = match self.optional_bracket_argument() {
            Some((raw, option_span)) => match raw.trim().parse::<usize>() {
                Ok(count) if count <= 9 => count,
                _ => {
                    self.diags.push(Diagnostic::error(
                        "\\newcolumntype argument count must be an integer from 0 to 9",
                        Some(option_span),
                        Some("ignored the definition".into()),
                    ));
                    let _ = self.required_group("newcolumntype", span);
                    return;
                }
            },
            None => 0,
        };
        let (body, _) = self.required_group("newcolumntype", span);
        if !self.array_package() {
            self.diags.push(Diagnostic::error(
                "\\newcolumntype needs the array package",
                Some(span),
                Some("recorded the column type anyway".into()),
            ));
        }
        self.column_types.insert(ch, (count, body));
    }

    /// `@{...}`: `\extracolsep` sets the `\tabskip` glue; anything else is
    /// text material in the template.
    fn at_expression(&mut self, tokens: Vec<InputToken>, pre: &mut Preamble, multicolumn: bool) {
        if let Some(rule) = self.rule_material(&tokens) {
            pre.add(rule);
            return;
        }
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
            if let SpecItem::Char(ch, span) = item {
                if let Some((count, definition)) = self.column_types.get(&ch).cloned() {
                    let mut arguments = Vec::with_capacity(count);
                    for _ in 0..count {
                        match items.next() {
                            Some(SpecItem::Group(argument, _)) => arguments.push(argument),
                            _ => {
                                self.diags.push(Diagnostic::error(
                                    format!("column type '{ch}' needs {count} braced arguments"),
                                    Some(span),
                                    Some("used empty arguments".into()),
                                ));
                                arguments.push(Vec::new());
                            }
                        }
                    }
                    if depth < MAX_SPEC_DEPTH {
                        let expanded = substitute_parameters(&definition, &arguments);
                        self.spec_items(&expanded, out, depth + 1);
                    }
                    continue;
                }
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

/// A `\newcolumntype` body with `#1`..`#9` replaced by the arguments.
fn substitute_parameters(body: &[InputToken], arguments: &[Vec<InputToken>]) -> Vec<InputToken> {
    let mut out = Vec::with_capacity(body.len());
    for input in body {
        let TokenKind::Word(word) = &input.token.kind else {
            out.push(input.clone());
            continue;
        };
        let bytes = word.as_bytes();
        let mut literal = String::new();
        let mut index = 0;
        let flush = |literal: &mut String, out: &mut Vec<InputToken>| {
            if !literal.is_empty() {
                out.push(InputToken {
                    token: Token {
                        kind: TokenKind::Word(std::mem::take(literal)),
                        span: input.token.span,
                    },
                    expansion_depth: input.expansion_depth,
                    maps_to_invocation: input.maps_to_invocation,
                });
            }
        };
        while index < bytes.len() {
            if bytes[index] == b'#' && index + 1 < bytes.len() && (b'1'..=b'9').contains(&bytes[index + 1]) {
                flush(&mut literal, &mut out);
                if let Some(argument) = arguments.get(usize::from(bytes[index + 1] - b'1')) {
                    out.extend(argument.iter().cloned());
                }
                index += 2;
            } else {
                let ch = word[index..].chars().next().expect("in bounds");
                literal.push(ch);
                index += ch.len_utf8();
            }
        }
        flush(&mut literal, &mut out);
    }
    out
}
