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

use flashtex_compiler::math::MathList;
use flashtex_compiler::parser::{Block as CBlock, Inline, Parsed};
use flashtex_compiler::Span;

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
    pub start: usize,
    pub end: usize,
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
    pub fn span(&self) -> Span {
        let start = self.segments.iter().flat_map(|s| s.chars.iter()).map(|c| c.start).min().unwrap_or(0);
        let end = self.segments.iter().flat_map(|s| s.chars.iter()).map(|c| c.end).max().unwrap_or(0);
        Span::new(start, end)
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParaPart {
    Lines(Vec<Item>),
    Display { list: MathList, span: Span },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Paragraph { parts: Vec<ParaPart>, indent: bool },
    Heading { level: u8, items: Vec<Item> },
}

#[derive(Debug)]
pub struct Doc {
    pub style: Stylesheet,
    pub blocks: Vec<Block>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Builds the block model for `source` from the compiler's parse result.
pub fn adapt(source: &str, parsed: &Parsed, options: &RenderOptions) -> Doc {
    let class_options = class_options(source).unwrap_or_else(|| options.default_class_options.clone());
    let size = class_size(&class_options);
    let parindent = parindent(source, size).or(options.default_parindent_pt).unwrap_or(match size {
        12 => 18.0,
        11 => 17.0,
        _ => 15.0,
    });
    let style = Stylesheet::from_document(&class_options, &parsed.packages, parindent);
    let styles = style_intervals(source);
    let mut blocks = Vec::new();
    let mut after_heading = false;
    for block in &parsed.blocks {
        match block {
            CBlock::Heading { level, content } => {
                let items = items_from_inlines(source, content, &styles);
                blocks.push(Block::Heading {
                    level: *level,
                    items,
                });
                after_heading = true;
            }
            CBlock::Paragraph(inlines) => {
                let items = items_from_inlines(source, inlines, &styles);
                let mut parts = Vec::new();
                let mut current = Vec::new();
                for item in items {
                    match item {
                        Item::Math { list, span } if is_display(inlines, span) => {
                            if !current.is_empty() {
                                parts.push(ParaPart::Lines(std::mem::take(&mut current)));
                            }
                            parts.push(ParaPart::Display { list, span });
                        }
                        other => current.push(other),
                    }
                }
                if !current.is_empty() {
                    parts.push(ParaPart::Lines(current));
                }
                if parts.is_empty() {
                    continue;
                }
                blocks.push(Block::Paragraph {
                    parts,
                    indent: !after_heading,
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

fn is_display(inlines: &[Inline], span: Span) -> bool {
    inlines.iter().any(|i| matches!(i, Inline::Math { display: true, span: s, .. } if *s == span))
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

fn style_at(intervals: &[(usize, usize, StyleKind)], at: usize) -> TextStyle {
    let mut s = TextStyle::default();
    for (start, end, kind) in intervals {
        if *start <= at && at < *end {
            match kind {
                StyleKind::Bold => s.bold = true,
                StyleKind::Italic => s.italic = true,
                StyleKind::Emph => s.italic = !s.italic,
            }
        }
    }
    s
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

/// TeX's space factor after `ch` (Appendix H: sfcodes of plain/LaTeX).
fn space_factor(ch: char, previous: u32) -> u32 {
    match ch {
        '.' | '?' | '!' => 3000,
        ',' => 1250,
        ';' | ':' => 1500,
        ')' | ']' | '\'' | '’' | '”' | '"' => previous,
        c if c.is_uppercase() => 999,
        _ => 1000,
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
fn items_from_inlines(source: &str, inlines: &[Inline], styles: &[(usize, usize, StyleKind)]) -> Vec<Item> {
    let mut items: Vec<Item> = Vec::new();
    let mut prev_end: Option<usize> = None;
    let mut prev_span: Option<Span> = None;
    let mut factor = 1000u32;
    let mut pending_accent: Option<(char, CharSrc)> = None;

    // Emits an interword space if the source between `prev` and `span` had one.
    let space_between = |prev_end: Option<usize>, prev_span: Option<Span>, span: Span| -> bool {
        match (prev_end, prev_span) {
            (None, _) => false,
            (Some(pe), Some(ps)) => {
                if ps == span {
                    // Same macro invocation span for both tokens: separate
                    // word tokens in a replacement text were space-separated.
                    true
                } else if pe <= span.start {
                    gap_has_space(&source[pe..span.start])
                } else {
                    false
                }
            }
            _ => false,
        }
    };

    for inline in inlines {
        match inline {
            Inline::LineBreak { span } => {
                items.push(Item::LineBreak);
                prev_end = Some(span.end);
                prev_span = Some(*span);
                factor = 1000;
            }
            Inline::Math { list, span, .. } => {
                if space_between(prev_end, prev_span, *span) {
                    items.push(Item::Space {
                        style: TextStyle::default(),
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
                let is_accent = span.end - span.start == 2
                    && source.as_bytes().get(span.start) == Some(&b'\\')
                    && text.chars().count() == 1
                    && "\"'`^~=.".contains(text.as_str());
                let style = style_at(styles, span.start);
                let has_space = space_between(prev_end, prev_span, *span);
                if has_space {
                    items.push(Item::Space {
                        style,
                        factor,
                        no_break: false,
                    });
                    pending_accent = None;
                }
                if is_accent {
                    pending_accent = Some((
                        text.chars().next().unwrap(),
                        CharSrc {
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
                let exact = span.end - span.start == text.len();
                let mut chars: Vec<(char, CharSrc)> = Vec::new();
                for (offset, ch) in text.char_indices() {
                    let src = if exact {
                        CharSrc {
                            start: span.start + offset,
                            end: span.start + offset + ch.len_utf8(),
                        }
                    } else {
                        CharSrc {
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
                let mut flush = |run: &mut Vec<(char, CharSrc)>, items: &mut Vec<Item>, factor: &mut u32| {
                    if run.is_empty() {
                        return;
                    }
                    let text: String = run.iter().map(|(c, _)| *c).collect();
                    let srcs: Vec<CharSrc> = run.iter().map(|(_, s)| *s).collect();
                    if let Some((last, _)) = run.last() {
                        *factor = space_factor(*last, *factor);
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
                prev_end = Some(span.end);
                prev_span = Some(*span);
            }
        }
    }
    items
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
fn tex_ligatures(chars: Vec<(char, CharSrc)>) -> Vec<(char, CharSrc)> {
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
                    start: s.start,
                    end: chars[i + n - 1].1.end,
                },
            )
        };
        if c == '-' && next == Some('-') && next2 == Some('-') {
            out.push(merged(3, '\u{2014}'));
            i += 3;
        } else if c == '-' && next == Some('-') {
            out.push(merged(2, '\u{2013}'));
            i += 2;
        } else if c == '`' && next == Some('`') {
            out.push(merged(2, '\u{201C}'));
            i += 2;
        } else if c == '\'' && next == Some('\'') {
            out.push(merged(2, '\u{201D}'));
            i += 2;
        } else if c == '`' {
            out.push((
                '\u{2018}',
                s,
            ));
            i += 1;
        } else if c == '\'' {
            out.push(('\u{2019}', s));
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
        let doc = adapt(src, &parsed, &RenderOptions::default());
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
        let doc = adapt(src, &flashtex_compiler::parser::parse(src), &RenderOptions::default());
        assert_eq!(doc.style.body_size_pt, 12.0);
        assert_eq!(doc.style.parindent_pt, 0.0);
    }

    #[test]
    fn space_factor_follows_sentence_punctuation() {
        let it = items("End. Next, more");
        let factors: Vec<u32> = it
            .iter()
            .filter_map(|i| match i {
                Item::Space { factor, .. } => Some(*factor),
                _ => None,
            })
            .collect();
        assert_eq!(factors, vec![3000, 1250]);
    }
}
