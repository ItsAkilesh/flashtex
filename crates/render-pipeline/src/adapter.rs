//! Adapter: compiler parse tree -> styled block model.
//!
//! The compiler's `parser::Parsed` (blocks of `Inline::Text` words with exact
//! byte spans, `Inline::Math` lists, `Inline::LineBreak`) drops information
//! this pipeline needs: which words were inside `\textbf`/`\emph`/`\textit`,
//! whether whitespace separated two words, the class options, `\parindent`,
//! and TeX's input conventions (`---`, quotes, `\'e`). Every one of those is
//! re-derived here from the exact source bytes the spans point into, which
//! is possible because the spans are exact. What the compiler lead is asked
//! to expose instead is listed in docs/proposals/rendering-abi.md
//! ("Requested compiler API").

use std::collections::BTreeMap;

use flashtex_compiler::math::MathList;
use flashtex_compiler::parser::{Block as CBlock, Inline, Parsed};
use flashtex_compiler::{DocumentId, Span};

use flashtex_document_style::{Geometry, Pt};

use crate::display::Diagnostic;
use crate::style::Stylesheet;
use crate::RenderOptions;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct TextStyle {
    pub bold: bool,
    pub italic: bool,
}

/// One output character and the source bytes it came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharSrc {
    pub document: DocumentId,
    pub start: usize,
    pub end: usize,
}

impl CharSrc {
    pub fn span(&self) -> Span {
        Span::in_document(self.document, self.start, self.end)
    }
}

/// A maximal run of characters in one style with no interword space.
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    pub text: String,
    /// One entry per `char` of `text`, in order.
    pub chars: Vec<CharSrc>,
    pub style: TextStyle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Word {
    pub segments: Vec<Segment>,
}

impl Word {
    /// Smallest span covering every character of the word, in the document
    /// of its first character (a word never straddles two documents).
    pub fn span(&self) -> Span {
        let document = self.segments.iter().flat_map(|s| s.chars.iter()).map(|c| c.document).next().unwrap_or_default();
        let start = self.segments.iter().flat_map(|s| s.chars.iter()).map(|c| c.start).min().unwrap_or(0);
        let end = self.segments.iter().flat_map(|s| s.chars.iter()).map(|c| c.end).max().unwrap_or(0);
        Span::in_document(document, start, end)
    }
    pub fn text(&self) -> String {
        self.segments.iter().map(|s| s.text.as_str()).collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Word(Word),
    /// Interword glue. `factor` is TeX's space factor (1000 normal, 3000
    /// after sentence-ending punctuation, 999 after an uppercase letter).
    Space { style: TextStyle, factor: u32, no_break: bool },
    Math { list: MathList, span: Span },
    /// `\\`
    LineBreak,
    /// Fixed horizontal glue of `em` ems of the current font (`\quad`
    /// after a section number).
    Quad { em: f64 },
    /// `\label{key}`: no material; records where the key's page is.
    Label { key: String },
    /// `\/` after a `\textit`/`\emph`/`\textbf` argument (LaTeX's
    /// `\text@command` adds it unless `.` or `,` follows).
    ItalicCorrection,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParaPart {
    Lines(Vec<Item>),
    /// A display; `number` is the `equation` counter text and the
    /// environment's source span (`\eqno` at the right margin).
    /// `bracket` marks LaTeX's `\[`/`displaymath`, which in vertical mode
    /// first sets an empty `.6\linewidth` box with `\nointerlineskip`.
    Display {
        list: MathList,
        span: Span,
        number: Option<(String, Span)>,
        bracket: bool,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Paragraph {
        parts: Vec<ParaPart>,
        indent: bool,
        /// `\newpage`/`\clearpage`/`\pagebreak` stood between the previous
        /// block and this one (the compiler reports and drops the command;
        /// the break is recovered from the source bytes).
        eject_before: bool,
    },
    Heading {
        level: u8,
        items: Vec<Item>,
        eject_before: bool,
    },
}

#[derive(Debug)]
pub struct Doc {
    pub style: Stylesheet,
    pub blocks: Vec<Block>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Label values (`\ref`) and the pages they fell on in a previous layout
/// pass (`\pageref`); a key absent from `pages` renders as `??`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Labels {
    pub values: BTreeMap<String, String>,
    pub pages: BTreeMap<String, u32>,
}

fn inlines_of(block: &CBlock) -> &[Inline] {
    match block {
        CBlock::Paragraph(i) => i,
        CBlock::Heading { content, .. } | CBlock::FigureCaption { content } => content,
    }
}

impl Labels {
    /// The `\ref` values of every `\label` in the parse (known before layout).
    pub fn from_parsed(parsed: &Parsed) -> Labels {
        let mut values = BTreeMap::new();
        for inline in parsed.blocks.iter().flat_map(inlines_of) {
            if let Inline::Label { key, value, .. } = inline {
                values.insert(key.clone(), value.clone());
            }
        }
        Labels {
            values,
            pages: BTreeMap::new(),
        }
    }

    /// Whether any `\pageref` in the parse needs a page number.
    pub fn needs_pages(parsed: &Parsed) -> bool {
        parsed
            .blocks
            .iter()
            .flat_map(inlines_of)
            .any(|i| matches!(i, Inline::Reference { page: true, .. }))
    }
}

/// Builds the block model from the compiler's parse result. `texts` is
/// indexed by `DocumentId`; `entry` is the root document's index.
pub fn adapt(texts: &[&str], entry: usize, parsed: &Parsed, options: &RenderOptions, labels: &Labels) -> Doc {
    let source = texts.get(entry).copied().unwrap_or("");
    let explicit_class = class_options(source);
    let class_options = explicit_class.clone().unwrap_or_else(|| options.default_class_options.clone());
    let size = class_size(&class_options);
    // LaTeX's own \parindent (size1x.clo) applies when the document declares a
    // class; body-only input keeps the compiler's implicit 0pt.
    let latex_parindent = match size {
        12 => 17.62482,
        11 => 16.5,
        _ => 15.0,
    };
    let parindent = parindent(source, size).unwrap_or(if explicit_class.is_some() {
        latex_parindent
    } else {
        options.default_parindent_pt
    });
    // Body-only input inherits the compiler's implicit preamble (1in margins);
    // a declared class uses article's own margins unless geometry says otherwise.
    let geometry = match package_options(source, "geometry") {
        Some(opts) => Some(Stylesheet::geometry_from_options(&opts)),
        None if explicit_class.is_none() => Some(Geometry::margin(Pt::inches(1.0))),
        None => None,
    };
    let style = Stylesheet::from_document(&class_options, &parsed.packages, geometry, parindent);
    let secnumdepth = counter(source, "secnumdepth").unwrap_or(options.default_secnumdepth);
    let styles: Vec<Styles> = texts.iter().map(|t| Styles::new(style_intervals(t))).collect();
    let mut blocks = Vec::new();
    let mut after_heading = false;
    for unit in split_at_page_breaks(texts, parsed) {
        let eject_before = unit.eject_before;
        match unit.kind {
            UnitKind::Heading {
                level,
                number,
                number_span,
                content,
            } => {
                // LaTeX `\@seccntformat`: the counter, then `\quad`, then the
                // title; the number's bytes are the `\section` command's.
                let mut items = Vec::new();
                if !number.is_empty() && level <= secnumdepth {
                    let chars = number
                        .chars()
                        .map(|_| CharSrc {
                            document: number_span.document,
                            start: number_span.start,
                            end: number_span.end,
                        })
                        .collect();
                    push_segment(&mut items, number.to_string(), chars, TextStyle::default());
                    items.push(Item::Quad { em: 1.0 });
                }
                items.extend(items_from_inlines(texts, content, &styles, labels));
                blocks.push(Block::Heading {
                    level,
                    items,
                    eject_before,
                });
                after_heading = true;
            }
            UnitKind::Paragraph { inlines, caption } => {
                let items = items_from_inlines(texts, inlines, &styles, labels);
                let mut parts = Vec::new();
                let mut current = Vec::new();
                for item in items {
                    match item {
                        Item::Math { list, span } if is_display(inlines, span) => {
                            if !current.is_empty() {
                                parts.push(ParaPart::Lines(std::mem::take(&mut current)));
                            }
                            // The compiler counts every closed display; LaTeX
                            // numbers only the `equation` environment.
                            let rest = texts.get(span.document.0).and_then(|t| t.get(span.start..)).unwrap_or("");
                            let number = display_number(inlines, span).filter(|_| rest.starts_with("\\begin{equation}"));
                            let bracket = rest.starts_with("\\[") || rest.starts_with("\\begin{displaymath}");
                            parts.push(ParaPart::Display {
                                list,
                                span,
                                number,
                                bracket,
                            });
                        }
                        other => current.push(other),
                    }
                }
                if !current.is_empty() {
                    parts.push(ParaPart::Lines(current));
                }
                let only_labels = parts
                    .iter()
                    .all(|p| matches!(p, ParaPart::Lines(items) if items.iter().all(|i| matches!(i, Item::Label { .. }))));
                if parts.is_empty() || only_labels {
                    continue;
                }
                blocks.push(Block::Paragraph {
                    parts,
                    indent: !after_heading && !caption,
                    eject_before,
                });
                after_heading = false;
            }
        }
    }
    Doc {
        style,
        blocks,
        diagnostics: Vec::new(),
    }
}

fn inline_span(i: &Inline) -> Span {
    match i {
        Inline::Text { span, .. }
        | Inline::LineBreak { span }
        | Inline::Math { span, .. }
        | Inline::Label { span, .. }
        | Inline::Reference { span, .. } => *span,
    }
}

/// A compiler block, or the piece of a paragraph between page-break
/// commands (`\newpage` ends the paragraph in LaTeX; the compiler keeps the
/// text in one block and reports the command as unsupported).
struct Unit<'p> {
    kind: UnitKind<'p>,
    eject_before: bool,
}

enum UnitKind<'p> {
    Heading {
        level: u8,
        number: &'p str,
        number_span: Span,
        content: &'p [Inline],
    },
    Paragraph {
        inlines: &'p [Inline],
        caption: bool,
    },
}

const PAGE_BREAKS: [&str; 3] = ["newpage", "clearpage", "pagebreak"];

/// Whether the source between `prev` and `next` (same document, in order)
/// holds a page-break command.
fn gap_has_page_break(texts: &[&str], prev: Span, next: Span) -> bool {
    if prev.document != next.document || prev.end > next.start {
        return false;
    }
    let gap = texts.get(next.document.0).and_then(|t| t.get(prev.end..next.start)).unwrap_or("");
    PAGE_BREAKS.iter().any(|c| find_command(gap, c).is_some())
}

fn split_at_page_breaks<'p>(texts: &[&str], parsed: &'p Parsed) -> Vec<Unit<'p>> {
    let mut units = Vec::new();
    let mut prev_end: Option<Span> = None;
    for block in &parsed.blocks {
        let first = match block {
            CBlock::Heading { number_span, .. } => Some(*number_span),
            _ => inlines_of(block).iter().map(inline_span).next(),
        };
        let mut eject = matches!((prev_end, first), (Some(p), Some(f)) if gap_has_page_break(texts, p, f));
        match block {
            CBlock::Heading {
                level,
                number,
                number_span,
                content,
            } => {
                units.push(Unit {
                    kind: UnitKind::Heading {
                        level: *level,
                        number,
                        number_span: *number_span,
                        content,
                    },
                    eject_before: eject,
                });
            }
            CBlock::Paragraph(inlines) | CBlock::FigureCaption { content: inlines } => {
                let caption = matches!(block, CBlock::FigureCaption { .. });
                let mut start = 0usize;
                for i in 1..inlines.len() {
                    if gap_has_page_break(texts, inline_span(&inlines[i - 1]), inline_span(&inlines[i])) {
                        units.push(Unit {
                            kind: UnitKind::Paragraph {
                                inlines: &inlines[start..i],
                                caption,
                            },
                            eject_before: eject,
                        });
                        eject = true;
                        start = i;
                    }
                }
                units.push(Unit {
                    kind: UnitKind::Paragraph {
                        inlines: &inlines[start..],
                        caption,
                    },
                    eject_before: eject,
                });
            }
        }
        if let Some(last) = inlines_of(block).iter().map(inline_span).last() {
            prev_end = Some(last);
        }
    }
    units
}

fn is_display(inlines: &[Inline], span: Span) -> bool {
    inlines.iter().any(|i| matches!(i, Inline::Math { display: true, span: s, .. } if *s == span))
}

fn display_number(inlines: &[Inline], span: Span) -> Option<(String, Span)> {
    inlines.iter().find_map(|i| match i {
        Inline::Math {
            display: true,
            span: s,
            number: Some(n),
            number_span,
            ..
        } if *s == span => Some((n.clone(), number_span.unwrap_or(span))),
        _ => None,
    })
}

/// Options of `\usepackage[opts]{name}`, if the package is loaded.
pub fn package_options(source: &str, name: &str) -> Option<String> {
    let mut from = 0;
    while let Some(at) = find_command(&source[from..], "usepackage") {
        let abs = from + at;
        let rest = source[abs + "\\usepackage".len()..].trim_start();
        let (opts, rest) = match rest.strip_prefix('[') {
            Some(inner) => {
                let end = inner.find(']')?;
                (inner[..end].to_string(), inner[end + 1..].trim_start())
            }
            None => (String::new(), rest),
        };
        if let Some(arg) = rest.strip_prefix('{') {
            if let Some(end) = arg.find('}') {
                if arg[..end].split(',').any(|p| p.trim() == name) {
                    return Some(opts);
                }
            }
        }
        from = abs + 1;
    }
    None
}

/// `\documentclass[opts]{...}` options, if the source has a class line.
pub fn class_options(source: &str) -> Option<String> {
    let at = find_command(source, "documentclass")?;
    let rest = &source[at + "\\documentclass".len()..];
    let rest = rest.trim_start();
    if let Some(inner) = rest.strip_prefix('[') {
        let end = inner.find(']')?;
        Some(inner[..end].to_string())
    } else {
        Some(String::new())
    }
}

pub fn class_size(options: &str) -> u32 {
    options
        .split(',')
        .filter_map(|o| o.trim().strip_suffix("pt"))
        .filter_map(|n| n.parse::<u32>().ok())
        .find(|n| matches!(n, 10 | 11 | 12))
        .unwrap_or(10)
}

/// `\setcounter{<name>}{<n>}`, the last one in the source.
pub fn counter(source: &str, name: &str) -> Option<u8> {
    let mut from = 0;
    let mut value = None;
    while let Some(at) = find_command(&source[from..], "setcounter") {
        let abs = from + at;
        let rest = source[abs + "\\setcounter".len()..].trim_start();
        if let Some(r) = rest.strip_prefix('{').and_then(|r| r.strip_prefix(name)).and_then(|r| r.strip_prefix('}')) {
            if let Some(r) = r.trim_start().strip_prefix('{') {
                if let Some(end) = r.find('}') {
                    value = r[..end].trim().parse::<u8>().ok().or(value);
                }
            }
        }
        from = abs + 1;
    }
    value
}

/// `\setlength{\parindent}{<dim>}` in points; `em` is resolved against the
/// body size.
pub fn parindent(source: &str, size: u32) -> Option<f64> {
    let mut from = 0;
    while let Some(at) = find_command(&source[from..], "setlength") {
        let abs = from + at;
        let rest = source[abs + "\\setlength".len()..].trim_start();
        if let Some(r) = rest.strip_prefix("{\\parindent}") {
            let r = r.trim_start().strip_prefix('{')?;
            let end = r.find('}')?;
            return parse_dimen(&r[..end], size);
        }
        from = abs + 1;
    }
    None
}

fn parse_dimen(s: &str, size: u32) -> Option<f64> {
    let s = s.trim();
    let split = s.find(|c: char| c.is_ascii_alphabetic())?;
    let (num, unit) = s.split_at(split);
    let v: f64 = num.trim().parse().ok()?;
    let body = match size {
        12 => 12.0,
        11 => 10.95,
        _ => 10.0,
    };
    Some(match unit.trim() {
        "pt" => v,
        "em" => v * body,
        "ex" => v * body * 0.430556,
        "in" => v * 72.27,
        "cm" => v * 72.27 / 2.54,
        "mm" => v * 72.27 / 25.4,
        "bp" => v * 72.27 / 72.0,
        _ => return None,
    })
}

/// Byte offset of `\name` (as a whole control word, outside comments).
fn find_command(source: &str, name: &str) -> Option<usize> {
    let needle = format!("\\{name}");
    let bytes = source.as_bytes();
    let mut i = 0;
    let mut in_comment = false;
    while i < bytes.len() {
        let c = bytes[i];
        if in_comment {
            if c == b'\n' {
                in_comment = false;
            }
            i += 1;
            continue;
        }
        if c == b'%' {
            in_comment = true;
            i += 1;
            continue;
        }
        if c == b'\\' {
            if source[i..].starts_with(&needle) {
                let after = i + needle.len();
                if after >= bytes.len() || !bytes[after].is_ascii_alphabetic() {
                    return Some(i);
                }
            }
            i += 2;
            continue;
        }
        i += 1;
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StyleKind {
    Bold,
    Emph,
    Italic,
}

/// Brace-group intervals of `\textbf{}`, `\emph{}`, `\textit{}` in source
/// byte offsets (content only), in document order.
fn style_intervals(source: &str) -> Vec<(usize, usize, StyleKind)> {
    let mut out = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0;
    let mut in_comment = false;
    while i < bytes.len() {
        let c = bytes[i];
        if in_comment {
            if c == b'\n' {
                in_comment = false;
            }
            i += 1;
            continue;
        }
        match c {
            b'%' => {
                in_comment = true;
                i += 1;
            }
            b'\\' => {
                let rest = &source[i..];
                let kind = if rest.starts_with("\\textbf") && !continues_word(bytes, i + 7) {
                    Some((StyleKind::Bold, 7))
                } else if rest.starts_with("\\emph") && !continues_word(bytes, i + 5) {
                    Some((StyleKind::Emph, 5))
                } else if rest.starts_with("\\textit") && !continues_word(bytes, i + 7) {
                    Some((StyleKind::Italic, 7))
                } else {
                    None
                };
                match kind {
                    Some((k, len)) => {
                        let mut j = i + len;
                        while j < bytes.len() && (bytes[j] as char).is_whitespace() {
                            j += 1;
                        }
                        if j < bytes.len() && bytes[j] == b'{' {
                            if let Some(close) = matching_brace(bytes, j) {
                                out.push((j + 1, close, k));
                            }
                        }
                        i += len;
                    }
                    None => i += 2,
                }
            }
            _ => i += 1,
        }
    }
    out
}

fn continues_word(bytes: &[u8], at: usize) -> bool {
    at < bytes.len() && bytes[at].is_ascii_alphabetic()
}

fn matching_brace(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 1,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// The style groups of one document, indexed for point queries: the
/// intervals in source order (sorted by start, properly nested), the
/// running maximum of their ends (so a backward scan can stop as soon as no
/// earlier group can still contain the position), and the ends sorted.
#[derive(Debug, Clone, Default)]
struct Styles {
    intervals: Vec<(usize, usize, StyleKind)>,
    max_end: Vec<usize>,
    ends: Vec<usize>,
}

impl Styles {
    fn new(intervals: Vec<(usize, usize, StyleKind)>) -> Styles {
        let mut max_end = Vec::with_capacity(intervals.len());
        let mut m = 0;
        for (_, end, _) in &intervals {
            m = m.max(*end);
            max_end.push(m);
        }
        let mut ends: Vec<usize> = intervals.iter().map(|(_, e, _)| *e).collect();
        ends.sort_unstable();
        Styles { intervals, max_end, ends }
    }

    /// The style in force at byte `at` (bold/italic set, emph toggles:
    /// order-independent, so the groups are visited from the nearest).
    fn at(&self, at: usize) -> TextStyle {
        let mut s = TextStyle::default();
        let p = self.intervals.partition_point(|(start, _, _)| *start <= at);
        let mut i = p;
        while i > 0 {
            i -= 1;
            if self.max_end[i] <= at {
                break;
            }
            let (_, end, kind) = self.intervals[i];
            if at < end {
                match kind {
                    StyleKind::Bold => s.bold = true,
                    StyleKind::Italic => s.italic = true,
                    StyleKind::Emph => s.italic = !s.italic,
                }
            }
        }
        s
    }

    /// Whether a style group's content ends exactly at `at`.
    fn closes_at(&self, at: usize) -> bool {
        self.ends.binary_search(&at).is_ok()
    }
}

fn style_at(styles: &Styles, at: usize) -> TextStyle {
    styles.at(at)
}

/// Whether the bytes between two consecutive inlines contain an interword
/// space under TeX's rules (braces and control words produce none; spaces
/// after a control word are eaten; comments swallow their newline).
fn gap_has_space(gap: &str) -> bool {
    let bytes = gap.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'{' | b'}' | b'[' | b']' => i += 1,
            b'%' => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                i += 1;
            }
            b'\\' => {
                i += 1;
                if i < bytes.len() && bytes[i].is_ascii_alphabetic() {
                    while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
                        i += 1;
                    }
                    while i < bytes.len() && (bytes[i] as char).is_whitespace() {
                        i += 1;
                    }
                } else if i < bytes.len() && bytes[i] == b' ' {
                    return true;
                } else {
                    i += 1;
                }
            }
            c if (c as char).is_whitespace() => return true,
            _ => i += 1,
        }
    }
    false
}

/// TeX's space factor after `ch` (§1034 with plain.tex's `\sfcode`s, which
/// LaTeX keeps): `.?!` 3000, `:` 2000, `;` 1500, `,` 1250, closing
/// delimiters and quotes 0 (keep), uppercase 999. A code above 1000 does
/// not take effect while the factor is below 1000 (after an uppercase
/// letter "A." keeps 1000), which is why the update runs per character.
fn space_factor(ch: char, previous: u32) -> u32 {
    let code = match ch {
        '.' | '?' | '!' => 3000,
        ':' => 2000,
        ';' => 1500,
        ',' => 1250,
        ')' | ']' | '\'' | '’' | '”' | '"' => 0,
        c if c.is_uppercase() => 999,
        _ => 1000,
    };
    if code == 1000 {
        1000
    } else if code < 1000 {
        if code > 0 {
            code
        } else {
            previous
        }
    } else if previous < 1000 {
        1000
    } else {
        code
    }
}

fn accent(mark: char, base: char) -> Option<char> {
    let table: &[(char, &str, &str)] = &[
        ('"', "aeiouyAEIOUY", "äëïöüÿÄËÏÖÜŸ"),
        ('\'', "aeiouyAEIOUYcnszCNSZ", "áéíóúýÁÉÍÓÚÝćńśźĆŃŚŹ"),
        ('`', "aeiouAEIOU", "àèìòùÀÈÌÒÙ"),
        ('^', "aeiouAEIOU", "âêîôûÂÊÎÔÛ"),
        ('~', "anoANO", "ãñõÃÑÕ"),
        ('=', "aeiouAEIOU", "āēīōūĀĒĪŌŪ"),
        ('.', "zcegZCEG", "żċėġŻĊĖĠ"),
    ];
    for (m, bases, composed) in table {
        if *m == mark {
            let idx = bases.chars().position(|b| b == base)?;
            return composed.chars().nth(idx);
        }
    }
    None
}

/// Converts the compiler inlines into words, spaces, math and line breaks.
/// `texts` and `styles` are indexed by `DocumentId`.
fn items_from_inlines(texts: &[&str], inlines: &[Inline], styles: &[Styles], labels: &Labels) -> Vec<Item> {
    // `\ref`/`\pageref` become ordinary text attributed to the command's
    // bytes; `\label` becomes a zero-width marker.
    let mut resolved: Vec<std::borrow::Cow<Inline>> = Vec::with_capacity(inlines.len());
    let mut reference_spans: Vec<Span> = Vec::new();
    for inline in inlines {
        match inline {
            Inline::Reference { key, page, span } => {
                let text = if *page {
                    labels.pages.get(key).map(|p| p.to_string())
                } else {
                    labels.values.get(key).cloned()
                }
                .unwrap_or_else(|| "??".to_string());
                reference_spans.push(*span);
                resolved.push(std::borrow::Cow::Owned(Inline::Text { text, span: *span }));
            }
            other => resolved.push(std::borrow::Cow::Borrowed(other)),
        }
    }
    let mut items: Vec<Item> = Vec::new();
    let mut prev_end: Option<usize> = None;
    let mut prev_span: Option<Span> = None;
    let mut factor = 1000u32;
    let mut pending_accent: Option<(char, CharSrc)> = None;
    let text_of = |d: DocumentId| -> &str { texts.get(d.0).copied().unwrap_or("") };
    let no_styles = Styles::default();
    let styles_of = |d: DocumentId| -> &Styles { styles.get(d.0).unwrap_or(&no_styles) };

    // Emits an interword space if the source between `prev` and `span` had one.
    let space_between = |prev_end: Option<usize>, prev_span: Option<Span>, span: Span| -> bool {
        match (prev_end, prev_span) {
            (None, _) => false,
            (Some(pe), Some(ps)) => {
                if ps == span {
                    // Same macro invocation span for both tokens: separate
                    // word tokens in a replacement text were space-separated.
                    true
                } else if ps.document != span.document {
                    // Crossing an \input boundary: TeX reads the newline that
                    // ends the \input line as a space.
                    true
                } else if pe <= span.start {
                    let src = text_of(span.document);
                    src.get(pe..span.start).is_some_and(gap_has_space)
                } else {
                    false
                }
            }
            _ => false,
        }
    };

    for inline in resolved.iter() {
        match &**inline {
            Inline::Label { key, .. } => items.push(Item::Label { key: key.clone() }),
            Inline::Reference { .. } => unreachable!("references were resolved above"),
            Inline::LineBreak { span } => {
                items.push(Item::LineBreak);
                prev_end = Some(span.end);
                prev_span = Some(*span);
                factor = 1000;
            }
            Inline::Math { list, span, .. } => {
                if space_between(prev_end, prev_span, *span) {
                    // The glue is the current font's where the space sits.
                    let gap_style = space_style(texts, styles, prev_end, *span, TextStyle::default());
                    items.push(Item::Space {
                        style: gap_style,
                        factor,
                        no_break: false,
                    });
                }
                items.push(Item::Math {
                    list: list.clone(),
                    span: *span,
                });
                prev_end = Some(span.end);
                prev_span = Some(*span);
                factor = 1000;
            }
            Inline::Text { text, span } => {
                let source = text_of(span.document);
                let is_accent = span.end - span.start == 2
                    && source.as_bytes().get(span.start) == Some(&b'\\')
                    && text.chars().count() == 1
                    && "\"'`^~=.".contains(text.as_str());
                let style = style_at(styles_of(span.document), span.start);
                let has_space = space_between(prev_end, prev_span, *span);
                if has_space {
                    // TeX sizes an interword space with the font current
                    // where the space token is read ("Plain, \textbf{bold}"
                    // gets a regular space, "\textbf{bold words}" a bold one,
                    // "\textbf{\emph{x}} y" a regular one).
                    let gap_style = space_style(texts, styles, prev_end, *span, style);
                    items.push(Item::Space {
                        style: gap_style,
                        factor,
                        no_break: false,
                    });
                    pending_accent = None;
                }
                if is_accent {
                    pending_accent = Some((
                        text.chars().next().unwrap(),
                        CharSrc {
                            document: span.document,
                            start: span.start,
                            end: span.end,
                        },
                    ));
                    prev_end = Some(span.end);
                    prev_span = Some(*span);
                    continue;
                }
                // Per-character sources. Macro replacement text shares the
                // invocation span; keep that attribution for every char.
                let exact = span.end - span.start == text.len() && !reference_spans.contains(span);
                let mut chars: Vec<(char, CharSrc)> = Vec::new();
                for (offset, ch) in text.char_indices() {
                    let src = if exact {
                        CharSrc {
                            document: span.document,
                            start: span.start + offset,
                            end: span.start + offset + ch.len_utf8(),
                        }
                    } else {
                        CharSrc {
                            document: span.document,
                            start: span.start,
                            end: span.end,
                        }
                    };
                    chars.push((ch, src));
                }
                if let Some((mark, msrc)) = pending_accent.take() {
                    if let Some((first, fsrc)) = chars.first().copied() {
                        if let Some(composed) = accent(mark, first) {
                            chars[0] = (
                                composed,
                                CharSrc {
                                    document: fsrc.document,
                                    start: msrc.start,
                                    end: fsrc.end,
                                },
                            );
                        }
                    }
                }
                let chars = tex_ligatures(chars);
                // `~` is an unbreakable space.
                let mut run: Vec<(char, CharSrc)> = Vec::new();
                let flush = |run: &mut Vec<(char, CharSrc)>, items: &mut Vec<Item>, factor: &mut u32| {
                    if run.is_empty() {
                        return;
                    }
                    let text: String = run.iter().map(|(c, _)| *c).collect();
                    let srcs: Vec<CharSrc> = run.iter().map(|(_, s)| *s).collect();
                    for (c, _) in run.iter() {
                        *factor = space_factor(*c, *factor);
                    }
                    push_segment(items, text, srcs, style);
                    run.clear();
                };
                for (ch, src) in chars {
                    if ch == '~' {
                        flush(&mut run, &mut items, &mut factor);
                        items.push(Item::Space {
                            style,
                            factor: 1000,
                            no_break: true,
                        });
                        factor = 1000;
                        continue;
                    }
                    run.push((ch, src));
                }
                flush(&mut run, &mut items, &mut factor);
                // A style group closing right after this text: LaTeX's
                // \text@command appends \/ (`\maybe@ic`) unless the next
                // token is in \nocorrlist (`,` and `.`) or the enclosing
                // font is itself slanted (`\fontdimen1 > 0`).
                if styles_of(span.document).closes_at(span.end)
                    && source.as_bytes().get(span.end) == Some(&b'}')
                    && !matches!(source.as_bytes().get(span.end + 1), Some(b'.') | Some(b','))
                    && !style_at(styles_of(span.document), span.end + 1).italic
                    && matches!(items.last(), Some(Item::Word(_)))
                {
                    items.push(Item::ItalicCorrection);
                }
                prev_end = Some(span.end);
                prev_span = Some(*span);
            }
        }
    }
    items
}

/// The style in force where TeX reads the space token between the previous
/// inline (ending at `prev_end`) and `span`: the first whitespace byte of
/// the gap, which sits inside or outside the closing braces around it.
/// `fallback` when the gap cannot be located.
fn space_style(
    texts: &[&str],
    styles: &[Styles],
    prev_end: Option<usize>,
    span: Span,
    fallback: TextStyle,
) -> TextStyle {
    let Some(pe) = prev_end else { return fallback };
    let Some(src) = texts.get(span.document.0) else { return fallback };
    let no_styles = Styles::default();
    let intervals = styles.get(span.document.0).unwrap_or(&no_styles);
    let Some(gap) = src.get(pe..span.start) else { return fallback };
    match gap.find(|c: char| c.is_whitespace()) {
        Some(off) => style_at(intervals, pe + off),
        None => style_at(intervals, pe),
    }
}

/// Appends a segment to the current word or starts a new word.
fn push_segment(items: &mut Vec<Item>, text: String, chars: Vec<CharSrc>, style: TextStyle) {
    let segment = Segment { text, chars, style };
    match items.last_mut() {
        Some(Item::Word(word)) => {
            if let Some(last) = word.segments.last_mut() {
                if last.style == style {
                    last.text.push_str(&segment.text);
                    last.chars.extend(segment.chars);
                    return;
                }
            }
            word.segments.push(segment);
        }
        _ => items.push(Item::Word(Word {
            segments: vec![segment],
        })),
    }
}

/// TeX input ligatures of T1-encoded text: `--` `---` ` `` `` `''` `'`.
/// Each is the T1 slot the font's ligature program would select, resolved
/// to a character through the declared encoding table (never a cast).
fn tex_ligatures(chars: Vec<(char, CharSrc)>) -> Vec<(char, CharSrc)> {
    use crate::ids::{Encoding, EncodingCode};
    let t1 = |code: EncodingCode| -> char { code.to_char(Encoding::T1).expect("declared T1 slot") };
    let mut out: Vec<(char, CharSrc)> = Vec::with_capacity(chars.len());
    let mut i = 0;
    while i < chars.len() {
        let (c, s) = chars[i];
        let next = chars.get(i + 1).map(|(c, _)| *c);
        let next2 = chars.get(i + 2).map(|(c, _)| *c);
        let merged = |n: usize, ch: char| -> (char, CharSrc) {
            (
                ch,
                CharSrc {
                    document: s.document,
                    start: s.start,
                    end: chars[i + n - 1].1.end,
                },
            )
        };
        if c == '-' && next == Some('-') && next2 == Some('-') {
            out.push(merged(3, t1(EncodingCode::T1_EMDASH)));
            i += 3;
        } else if c == '-' && next == Some('-') {
            out.push(merged(2, t1(EncodingCode::T1_ENDASH)));
            i += 2;
        } else if c == '`' && next == Some('`') {
            out.push(merged(2, t1(EncodingCode::T1_QUOTEDBLLEFT)));
            i += 2;
        } else if c == '\'' && next == Some('\'') {
            out.push(merged(2, t1(EncodingCode::T1_QUOTEDBLRIGHT)));
            i += 2;
        } else if c == '`' {
            out.push((t1(EncodingCode::T1_QUOTELEFT), s));
            i += 1;
        } else if c == '\'' {
            out.push((t1(EncodingCode::T1_QUOTERIGHT), s));
            i += 1;
        } else {
            out.push((c, s));
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items(src: &str) -> Vec<Item> {
        let parsed = flashtex_compiler::parser::parse(src);
        let doc = adapt(&[src], 0, &parsed, &RenderOptions::default(), &Labels::default());
        match &doc.blocks[0] {
            Block::Paragraph { parts, .. } => match &parts[0] {
                ParaPart::Lines(items) => items.clone(),
                _ => panic!(),
            },
            Block::Heading { items, .. } => items.clone(),
        }
    }

    #[test]
    fn accents_and_dashes_compose_with_exact_sources() {
        let src = "Na\\\"ive caf\\'e --- dash.";
        let it = items(src);
        let words: Vec<String> = it
            .iter()
            .filter_map(|i| match i {
                Item::Word(w) => Some(w.text()),
                _ => None,
            })
            .collect();
        assert_eq!(words, vec!["Naïve", "café", "—", "dash."]);
        if let Item::Word(w) = &it[0] {
            let c = &w.segments[0].chars[2];
            assert_eq!(&src[c.start..c.end], "\\\"i");
        }
        if let Item::Word(w) = &it[4] {
            let c = &w.segments[0].chars[0];
            assert_eq!(&src[c.start..c.end], "---");
        }
    }

    #[test]
    fn styles_and_gaps_are_recovered_from_source() {
        let src = "Plain, \\textbf{bold}, \\emph{em\\emph{up}} and \\textbf{\\emph{bi}}.";
        let it = items(src);
        let mut seen = Vec::new();
        for i in &it {
            match i {
                Item::Word(w) => {
                    for s in &w.segments {
                        seen.push((s.text.clone(), s.style.bold, s.style.italic));
                    }
                }
                Item::Space { .. } => seen.push((" ".into(), false, false)),
                _ => {}
            }
        }
        assert_eq!(
            seen,
            vec![
                ("Plain,".to_string(), false, false),
                (" ".into(), false, false),
                ("bold".into(), true, false),
                (",".into(), false, false),
                (" ".into(), false, false),
                ("em".into(), false, true),
                ("up".into(), false, false),
                (" ".into(), false, false),
                ("and".into(), false, false),
                (" ".into(), false, false),
                ("bi".into(), true, true),
                (".".into(), false, false),
            ]
        );
    }

    #[test]
    fn class_options_and_parindent_are_read_from_source() {
        let src = "\\documentclass[12pt]{article}\n\\setlength{\\parindent}{0pt}\n\\begin{document}x\\end{document}";
        assert_eq!(class_options(src).as_deref(), Some("12pt"));
        assert_eq!(parindent(src, 12), Some(0.0));
        let doc = adapt(&[src], 0, &flashtex_compiler::parser::parse(src), &RenderOptions::default(), &Labels::default());
        assert_eq!(doc.style.body_size_pt, 12.0);
        assert_eq!(doc.style.parindent_pt, 0.0);
        let src2 = "\\documentclass{article}\n\\begin{document}x\\end{document}";
        let doc2 = adapt(&[src2], 0, &flashtex_compiler::parser::parse(src2), &RenderOptions::default(), &Labels::default());
        assert_eq!(doc2.style.body_size_pt, 10.0);
        assert_eq!(doc2.style.parindent_pt, 15.0);
    }

    #[test]
    fn space_factor_follows_sentence_punctuation() {
        let it = items("End. Next, more: A. B; (c.) d");
        let factors: Vec<u32> = it
            .iter()
            .filter_map(|i| match i {
                Item::Space { factor, .. } => Some(*factor),
                _ => None,
            })
            .collect();
        // "A." and "B;" stay 1000 (§1034: a code above 1000 after an uppercase
        // letter), ")" keeps the factor of the "." before it.
        assert_eq!(factors, vec![3000, 1250, 2000, 1000, 1000, 3000]);
    }
}
