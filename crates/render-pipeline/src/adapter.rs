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
    /// Font size set by a size declaration (`\Large`, ...) in force, in
    /// hundredths of a point; 0 keeps the paragraph's size.
    pub size_cpt: u16,
    /// `\normalfont`/`\mdseries` in force inside a heading: the block's
    /// own weight (`\bfseries` from `\@startsection`) is not applied.
    pub medium: bool,
}

impl TextStyle {
    /// The size to shape at, given the paragraph's `size`.
    pub fn size_or(self, size: f64) -> f64 {
        if self.size_cpt == 0 {
            size
        } else {
            f64::from(self.size_cpt) / 100.0
        }
    }
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
    /// `\\`; `skip_pt` is the optional `[<dimen>]` (LaTeX `\@xnewline`:
    /// `\vadjust{\vskip <dimen>}` after the line, or `\vskip` after the
    /// paragraph under `\@centercr`).
    LineBreak { skip_pt: f64 },
    /// Fixed horizontal glue of `em` ems of the current font (`\quad`
    /// after a section number).
    Quad { em: f64 },
    /// `\label{key}`: no material; records where the key's page is.
    Label { key: String },
    /// `\/` after a `\textit`/`\emph`/`\textbf` argument (LaTeX's
    /// `\text@command` adds it unless `.` or `,` follows).
    ItalicCorrection,
    /// `\hfill`/`\hfil` (compiler `Inline::HFill`): infinitely stretchable
    /// glue; a legal break point that is discarded at a line break. `fill`
    /// is the `\hfill` order (it beats `\parfillskip`'s `fil`); the
    /// compiler does not distinguish the two, so the order is re-read from
    /// the source bytes (`\hfill` when they are not `\hfil`).
    HFill { fill: bool },
    /// `\hspace{<dimen>}` (compiler `Inline::HSpace`): fixed glue in points.
    HSpace { pt: f64 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParaPart {
    Lines(Vec<Item>),
    /// A display; `number` is the `equation` counter text and the
    /// environment's source span (`\eqno` at the right margin).
    /// `bracket` marks LaTeX's `\[`/`displaymath`, which in vertical mode
    /// first sets an empty `.6\linewidth` box with `\nointerlineskip`.
    /// Under amsmath `\[` is `\begin{equation*}`, whose `\mathdisplay`
    /// is a bare `$$` (no box, no `\nointerlineskip`), so it is `false`
    /// there.
    Display {
        list: MathList,
        span: Span,
        number: Option<(String, Span)>,
        bracket: bool,
    },
}

/// LaTeX paragraph-shape environments (compiler `Block::Styled`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ParaStyle {
    #[default]
    Plain,
    /// `center`: `\centering` (`\leftskip`/`\rightskip` `0pt plus 1fil`,
    /// `\parfillskip 0pt`).
    Center,
    /// `flushleft`: `\raggedright`.
    FlushLeft,
    /// `flushright`: `\raggedleft`.
    FlushRight,
    /// `quote`/`quotation`: a level-1 list with `\rightmargin=\leftmargin`.
    Quote,
}

impl ParaStyle {
    fn of(style: flashtex_compiler::parser::ParagraphStyle) -> ParaStyle {
        use flashtex_compiler::parser::ParagraphStyle as P;
        match style {
            P::Center => ParaStyle::Center,
            P::FlushLeft => ParaStyle::FlushLeft,
            P::FlushRight => ParaStyle::FlushRight,
            P::Quote => ParaStyle::Quote,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Paragraph {
        parts: Vec<ParaPart>,
        indent: bool,
        style: ParaStyle,
        /// The paragraph opens its `center`/`quote`/... environment
        /// (`\trivlist`/`\list`: `\topsep` glue before it, plus
        /// `\partopsep` when the environment began in vertical mode) and/or
        /// closes it (`\@endparenv`: the same glue after it).
        env_open: Option<EnvOpen>,
        env_close: bool,
        /// `\newpage`/`\clearpage`/`\pagebreak` stood between the previous
        /// block and this one (the compiler reports and drops the command;
        /// the break is recovered from the source bytes).
        eject_before: bool,
        /// `\vspace{<dimen>}` blocks between the previous block and this
        /// one (compiler `Block::VSpace`), summed in points; `\addvspace`
        /// glue added before the block.
        vspace_before: f64,
        /// LaTeX `\addvspace` glue before the block (`\@item`'s `\topsep`/
        /// `\itemsep`, `\@endparenv`'s `\@topsepadd`), in points: only
        /// its excess over the previous block's trailing skip (a display's
        /// `\belowdisplayskip`) is added.
        addvspace_before: f64,
        /// `\endtrivlist` of the list(s) closed between the previous block
        /// and this one: when the previous block left a positive trailing
        /// skip (a display's `\belowdisplayskip`), each closing list
        /// changes it by its `\parsep` minus the `\parskip` outside it,
        /// in points; summed innermost first. Nothing when there was no
        /// trailing skip.
        endlist_adjust: f64,
        /// The paragraph is (part of) an `itemize`/`enumerate` `\item`
        /// (compiler `Block::ListItem`): LaTeX's `\list` geometry applies.
        list: Option<ListGeom>,
    },
    Heading {
        level: u8,
        items: Vec<Item>,
        eject_before: bool,
        vspace_before: f64,
    },
    /// `\hrule` in vertical mode: a full-measure rule 0.4pt high with no
    /// interline glue on either side (TeX §1056 sets `prev_depth` to
    /// `ignore_depth`).
    Rule {
        span: Span,
        eject_before: bool,
        vspace_before: f64,
    },
}

/// LaTeX `\list` geometry of one `\item` paragraph (see
/// [`Block::Paragraph::list`]). `\list` sets `\parshape` so every line of
/// the item starts `\@totalleftmargin` (the sum of the enclosing lists'
/// `\leftmargin`s) in from the left margin, and `\@item` sets the label
/// right-aligned in `\hbox to\labelwidth{\hss <label>}\hskip\labelsep`
/// before the first line, so its right edge ends `\labelsep` before the
/// text (article's `\makelabel` is `\hss\llap{#1}`, so a wider label
/// simply extends further left).
#[derive(Debug, Clone, PartialEq)]
pub struct ListGeom {
    /// Nesting level (1 = outermost).
    pub level: u8,
    /// `\leftmargin` of every enclosing list, outermost first; the hanging
    /// indent is their sum.
    pub margins: Vec<ListMargin>,
    /// The `\item` marker text and the command's span; `None` for a later
    /// paragraph of the same item (a blank line inside the item's text).
    pub label: Option<(String, Span)>,
    /// The innermost list's `\parsep` (`\list` sets `\parskip\parsep`):
    /// the glue every paragraph of the item adds. Article's `\@list<i>`
    /// value for the nesting level, or an enumitem `parsep=` key.
    pub parsep: crate::style::Skip,
}

/// One list level's `\leftmargin`.
#[derive(Debug, Clone, PartialEq)]
pub enum ListMargin {
    /// article's `\leftmargin<i>` (or an explicit enumitem
    /// `leftmargin=<dimen>`), in points.
    Fixed(f64),
    /// enumitem `leftmargin=*`: `\labelwidth` + `\labelsep`, where
    /// `\labelwidth` is the width of this label — the widest one the list
    /// can produce (enumitem's `widest` default: `m`/`M`/`viii`/`VIII`/`0`
    /// for `\alph`/`\Alph`/`\roman`/`\Roman`/`\arabic`), set in the
    /// body font.
    Widest(String),
}

/// How a paragraph-shape environment began (see [`Block::Paragraph`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnvOpen {
    /// `\begin{...}` was read in vertical mode (after a blank line, a
    /// heading, a rule or at the document start): `\partopsep` is added.
    pub vmode: bool,
}

#[derive(Debug)]
pub struct Doc {
    pub style: Stylesheet,
    pub blocks: Vec<Block>,
    pub diagnostics: Vec<Diagnostic>,
    /// Compiler constructs this pipeline has no exact block for and set
    /// approximately or dropped: `(code, source span, message)`, reported
    /// as warnings against the document paths by the caller.
    pub limitations: Vec<(&'static str, Span, String)>,
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
        CBlock::ListItem { content, .. } | CBlock::Heading { content, .. } | CBlock::FigureCaption { content } | CBlock::Styled { content, .. } => content,
        // `\maketitle`'s parts are lowered to `Styled` paragraphs before
        // the block walk (`lower_blocks`); only the title is visible here.
        CBlock::TitleBlock { title, .. } => title,
        CBlock::VSpace { .. } | CBlock::Rule { .. } | CBlock::PageBreak | CBlock::Verbatim { .. } | CBlock::TableOfContents { .. } | CBlock::VFill => &[],
    }
}

/// Rewrites the compiler blocks the pipeline has no layout for (pin
/// `d416472a`: `Verbatim`, `TableOfContents`, `TitleBlock`, `VFill`) into
/// the plain blocks it does set, with one typed `unsupported_block`
/// limitation each, so their content is never dropped:
///
/// - `verbatim`/`lstlisting`: a `flushleft` paragraph, one `Text` per
///   source line (the compiler's tab-expanded text, `Mono` family) joined
///   by `\\`; the pipeline sets it in the body face, justified off, so
///   the lines keep their order but not Courier's fixed pitch.
/// - `\tableofcontents`: article's own `\section*{\contentsname}` heading
///   without the entries (the page builder has no contents pass).
/// - `\maketitle`: three centred paragraphs (title at `\LARGE`, authors
///   and date at `\large`, the sizes `\@maketitle` declares) carried by the
///   compiler's `TextStyle::size`, which the pipeline already reads; the
///   exact `\vskip`s and `\thanks` are not.
/// - `\vfill`: dropped (the page builder has no stretchable vertical
///   glue), reported on the next block.
fn lower_blocks(texts: &[&str], blocks: &[CBlock]) -> (Vec<CBlock>, Vec<(&'static str, Span, String)>) {
    use flashtex_compiler::parser::{FontSizeLevel, ParagraphStyle, TextFamily, TextStyle as CStyle};
    let mut out: Vec<CBlock> = Vec::with_capacity(blocks.len());
    let mut limitations: Vec<(&'static str, Span, String)> = Vec::new();
    let mut pending_vfill = 0usize;
    let sized = |inlines: &[Inline], size: FontSizeLevel| -> Vec<Inline> {
        inlines
            .iter()
            .map(|i| match i {
                Inline::Text { text, span, style, space_before } => Inline::Text {
                    text: text.clone(),
                    span: *span,
                    style: CStyle {
                        size: Some(style.size.unwrap_or(size)),
                        ..*style
                    },
                    space_before: *space_before,
                },
                other => other.clone(),
            })
            .collect()
    };
    for block in blocks {
        let first = match block {
            CBlock::Heading { number_span, .. } => Some(*number_span),
            CBlock::Verbatim { span, .. } | CBlock::TableOfContents { span } | CBlock::Rule { span } => Some(*span),
            _ => inlines_of(block).iter().map(inline_span).next(),
        };
        if pending_vfill > 0 {
            if let Some(at) = first {
                limitations.push((
                    "unsupported_block",
                    at,
                    format!("\\vfill ({pending_vfill} before this block) dropped: the page builder has no stretchable vertical glue"),
                ));
                pending_vfill = 0;
            }
        }
        match block {
            CBlock::Verbatim { lines, span } => {
                let mut content: Vec<Inline> = Vec::with_capacity(lines.len() * 2);
                for (i, line) in lines.iter().enumerate() {
                    if i > 0 {
                        // The break owns the bytes between the lines so no
                        // interword space is read across it.
                        let prev = lines[i - 1].span;
                        content.push(Inline::LineBreak {
                            span: Span {
                                document: line.span.document,
                                start: prev.end.min(line.span.start),
                                end: line.span.start,
                            },
                        });
                    }
                    content.push(Inline::Text {
                        text: line.text.clone(),
                        span: line.span,
                        style: CStyle {
                            family: TextFamily::Mono,
                            ..CStyle::default()
                        },
                        space_before: true,
                    });
                }
                limitations.push((
                    "unsupported_block",
                    *span,
                    format!("verbatim ({} line(s)) set as a flush-left paragraph in the body face with forced line breaks: the pipeline has no monospaced face or literal-text block", lines.len()),
                ));
                out.push(CBlock::Styled {
                    style: ParagraphStyle::FlushLeft,
                    content,
                });
            }
            CBlock::TableOfContents { span } => {
                limitations.push((
                    "unsupported_block",
                    *span,
                    "\\tableofcontents set as its `Contents` heading only: the pipeline has no contents pass (entries and page numbers omitted)".to_string(),
                ));
                out.push(CBlock::Heading {
                    level: 1,
                    number: String::new(),
                    number_span: *span,
                    content: vec![Inline::Text {
                        text: "Contents".to_string(),
                        span: *span,
                        style: CStyle::BOLD,
                        space_before: true,
                    }],
                });
            }
            CBlock::TitleBlock { title, authors, date } => {
                if let Some(at) = first {
                    limitations.push((
                        "unsupported_block",
                        at,
                        "\\maketitle set as centred paragraphs (title \\LARGE, authors/date \\large): article's exact \\@maketitle skips and \\thanks are not applied".to_string(),
                    ));
                }
                for (part, size) in [(Some(title), FontSizeLevel::Large3), (Some(authors), FontSizeLevel::Large1), (date.as_ref(), FontSizeLevel::Large1)] {
                    let Some(part) = part else { continue };
                    if part.is_empty() {
                        continue;
                    }
                    out.push(CBlock::Styled {
                        style: ParagraphStyle::Center,
                        content: sized(part, size),
                    });
                }
            }
            CBlock::VFill => pending_vfill += 1,
            other => out.push(other.clone()),
        }
    }
    if pending_vfill > 0 {
        let at = out.iter().rev().flat_map(|b| inlines_of(b).iter().map(inline_span).last()).next().unwrap_or(Span {
            document: DocumentId(0),
            start: 0,
            end: 0,
        });
        let _ = texts;
        limitations.push((
            "unsupported_block",
            at,
            format!("\\vfill ({pending_vfill} at the end of the document) dropped: the page builder has no stretchable vertical glue"),
        ));
    }
    (out, limitations)
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
    adapt_cached(texts, entry, parsed, options, labels, None)
}

/// [`adapt`] with the cross-request cache: a compiler block whose inlines,
/// source bytes, enclosing style and label table match an earlier request
/// reuses its items (offsets relocated).
pub fn adapt_cached(
    texts: &[&str],
    entry: usize,
    parsed: &Parsed,
    options: &RenderOptions,
    labels: &Labels,
    cache: Option<&crate::incremental::RenderCache>,
) -> Doc {
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
    let mut style = Stylesheet::from_document(&class_options, &parsed.packages, geometry, parindent);
    // amsmath makes `\[` a plain `$$` (see [`ParaPart::Display::bracket`]).
    let amsmath = parsed.packages.iter().any(|p| p == "amsmath");
    // `\setlength{\parskip}{...}`: a fixed skip (no stretch) replaces
    // article's `0pt plus 1pt`.
    if let Some(pt) = parskip(source, size) {
        style.parskip = crate::style::Skip::fixed(pt);
    }
    let secnumdepth = counter(source, "secnumdepth").unwrap_or(options.default_secnumdepth);
    let styles: Vec<Styles> = texts.iter().map(|t| Styles::new(style_intervals(t))).collect();
    let labels_fp = {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        for (k, v) in &labels.values {
            k.hash(&mut h);
            v.hash(&mut h);
        }
        for (k, v) in &labels.pages {
            k.hash(&mut h);
            v.hash(&mut h);
        }
        h.finish()
    };
    let items_for = |inlines: &[Inline], heading: bool| -> Vec<Item> { items_cached(texts, inlines, &styles, labels, labels_fp, size, heading, cache) };
    let mut blocks = Vec::new();
    let (lowered, mut limitations) = lower_blocks(texts, &parsed.blocks);
    let mut after_heading = false;
    for unit in split_at_page_breaks(texts, &lowered, size, &style) {
        let eject_before = unit.eject_before;
        let vspace_before = unit.vspace_before;
        limitations.extend(unit.limitations);
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
                items.extend(items_for(content, true));
                blocks.push(Block::Heading {
                    level,
                    items,
                    eject_before,
                    vspace_before,
                });
                after_heading = true;
            }
            UnitKind::Rule { span } => {
                blocks.push(Block::Rule {
                    span,
                    eject_before,
                    vspace_before,
                });
                after_heading = false;
            }
            UnitKind::Paragraph {
                inlines,
                caption,
                styled,
                env_open,
                after_env,
                list,
            } => {
                for inline in inlines {
                    if let Inline::MathRows { rows, aligned, span } = inline {
                        let env = if *aligned { "align" } else { "gather" };
                        limitations.push((
                            "math_limitation",
                            *span,
                            format!("{env}: {} row(s) set as separate centred displays; `&` alignment points ignored (no multi-row display block yet)", rows.len()),
                        ));
                    }
                    unsupported_inlines(inline, &mut limitations);
                }
                let items = items_for(inlines, false);
                let mut parts = Vec::new();
                let mut current = Vec::new();
                for item in items {
                    match item {
                        Item::Math { list, span } if is_display(inlines, span) => {
                            if !current.is_empty() {
                                parts.push(ParaPart::Lines(std::mem::take(&mut current)));
                            }
                            // The compiler counts every closed display; LaTeX
                            // numbers only the `equation` environment. Rows of
                            // an amsmath display carry their own numbers.
                            let rest = texts.get(span.document.0).and_then(|t| t.get(span.start..)).unwrap_or("");
                            let number = match math_row_number(inlines, span) {
                                Some(row) => row,
                                None => display_number(inlines, span).filter(|_| rest.starts_with("\\begin{equation}")),
                            };
                            let bracket = !amsmath && (rest.starts_with("\\[") || rest.starts_with("\\begin{displaymath}"));
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
                // `\centering` sets `\parindent 0pt`; a list item's first
                // paragraph carries no indent and `\list` sets
                // `\parindent\listparindent` (0pt in article) for the
                // ones after it, `quote` likewise.
                blocks.push(Block::Paragraph {
                    parts,
                    indent: !after_heading && !caption && styled.is_none() && !after_env && list.is_none(),
                    style: styled.unwrap_or_default(),
                    env_open,
                    env_close: false,
                    eject_before,
                    vspace_before,
                    addvspace_before: unit.addvspace_before,
                    endlist_adjust: unit.endlist_adjust,
                    list,
                });
                after_heading = false;
            }
        }
    }
    // `\end{...}`: the last paragraph of a run of same-style paragraphs
    // closes the environment (two adjacent environments of one style are
    // read as one; the compiler does not mark the boundary).
    let styles: Vec<ParaStyle> = blocks
        .iter()
        .map(|b| match b {
            Block::Paragraph { style, .. } => *style,
            _ => ParaStyle::Plain,
        })
        .collect();
    for (i, block) in blocks.iter_mut().enumerate() {
        if let Block::Paragraph { style, env_close, .. } = block {
            if *style != ParaStyle::Plain {
                *env_close = styles.get(i + 1).is_none_or(|next| *next != *style);
            }
        }
    }
    Doc {
        style,
        blocks,
        diagnostics: Vec::new(),
        limitations,
    }
}

fn inline_span(i: &Inline) -> Span {
    match i {
        Inline::Text { span, .. }
        | Inline::LineBreak { span }
        | Inline::Math { span, .. }
        | Inline::MathRows { span, .. }
        | Inline::Label { span, .. }
        | Inline::Reference { span, .. }
        | Inline::HFill { span }
        | Inline::HSpace { span, .. }
        | Inline::Footnote { span, .. }
        | Inline::Verbatim { span, .. }
        | Inline::TextGlue { span, .. } => *span,
        Inline::Tabular(t) => t.span,
    }
}

/// The compiler inlines (pin `d416472a`) the pipeline sets only as plain
/// text, as `unsupported_block` limitations: `\footnote` (mark and text
/// inline, no page-bottom note), `tabular` (cells in reading order, no
/// columns or rules) and `\verb` (body face). Footnote text is scanned too.
fn unsupported_inlines(inline: &Inline, out: &mut Vec<(&'static str, Span, String)>) {
    match inline {
        Inline::Footnote { number, span, mark, text, .. } => {
            out.push((
                "unsupported_block",
                *span,
                format!(
                    "\\footnote {number}: {}{} set inline in the paragraph (no page-bottom footnote area yet)",
                    if *mark { "mark as plain text" } else { "no mark" },
                    if text.is_some() { ", note text" } else { "" }
                ),
            ));
            for i in text.iter().flatten() {
                unsupported_inlines(i, out);
            }
        }
        Inline::Tabular(t) => {
            let rows = t.entries.iter().filter(|e| matches!(e, flashtex_compiler::tabular::Entry::Row(_))).count();
            out.push((
                "unsupported_block",
                t.span,
                format!("tabular ({rows} row(s), {} column(s)) set as its cells' text in reading order: the pipeline has no table layout (columns, rules and alignment omitted)", t.columns.len()),
            ));
            for list in t.inline_lists() {
                for i in list {
                    unsupported_inlines(i, out);
                }
            }
        }
        Inline::Verbatim { text, span, .. } => {
            out.push(("unsupported_block", *span, format!("\\verb {text:?} set in the body face: the pipeline has no monospaced face")));
        }
        _ => {}
    }
}

/// Lowers one compiler inline into the shapes `items_from_inlines` sets:
/// `\ref`/`\pageref`/`\eqref` become text from the label table; a
/// footnote becomes its mark (plain text) followed by its note text; a
/// tabular becomes its cells' inlines in reading order with `\\` between
/// rows; `\verb` becomes `Mono` text. Everything else is borrowed.
fn lower_inline<'a>(inline: &'a Inline, labels: &Labels, reference_spans: &mut Vec<Span>, out: &mut Vec<std::borrow::Cow<'a, Inline>>) {
    use flashtex_compiler::parser::{TextFamily, TextStyle as CStyle};
    match inline {
        Inline::Reference { key, page, equation, span, .. } => {
            let text = if *page {
                labels.pages.get(key).map(|p| p.to_string())
            } else {
                labels.values.get(key).cloned()
            }
            .unwrap_or_else(|| "??".to_string());
            // amsmath `\eqref`: the value in parentheses (compiler's flag).
            let text = if *equation { format!("({text})") } else { text };
            reference_spans.push(*span);
            out.push(std::borrow::Cow::Owned(Inline::Text {
                text,
                span: *span,
                style: Default::default(),
                // Interword gaps are read from the source bytes between
                // spans here, never from the compiler's flag.
                space_before: true,
            }));
        }
        Inline::Footnote { number, span, mark, text, .. } => {
            if *mark {
                reference_spans.push(*span);
                out.push(std::borrow::Cow::Owned(Inline::Text {
                    text: number.clone(),
                    span: *span,
                    style: Default::default(),
                    space_before: true,
                }));
            }
            for i in text.iter().flatten() {
                lower_inline(i, labels, reference_spans, out);
            }
        }
        Inline::Tabular(t) => {
            use flashtex_compiler::tabular::Entry;
            let mut prev_row: Option<Span> = None;
            for entry in &t.entries {
                let Entry::Row(row) = entry else { continue };
                let first = row.cells.iter().flat_map(|c| c.content.iter().map(inline_span)).next();
                if let (Some(prev), Some(first)) = (prev_row, first) {
                    out.push(std::borrow::Cow::Owned(Inline::LineBreak {
                        span: Span {
                            document: first.document,
                            start: prev.end.min(first.start),
                            end: first.start,
                        },
                    }));
                }
                for cell in &row.cells {
                    for i in &cell.content {
                        lower_inline(i, labels, reference_spans, out);
                    }
                }
                if let Some(last) = row.cells.iter().flat_map(|c| c.content.iter().map(inline_span)).last() {
                    prev_row = Some(last);
                }
            }
        }
        Inline::Verbatim { text, span, space_before } => {
            reference_spans.push(*span);
            out.push(std::borrow::Cow::Owned(Inline::Text {
                text: text.clone(),
                span: *span,
                style: CStyle {
                    family: TextFamily::Mono,
                    ..CStyle::default()
                },
                space_before: *space_before,
            }));
        }
        other => out.push(std::borrow::Cow::Borrowed(other)),
    }
}

/// A compiler block, or the piece of a paragraph between page-break
/// commands (`\newpage` ends the paragraph in LaTeX; the compiler keeps the
/// text in one block and reports the command as unsupported).
struct Unit<'p> {
    kind: UnitKind<'p>,
    eject_before: bool,
    /// Summed `\vspace` points from compiler `VSpace` blocks before this unit.
    vspace_before: f64,
    /// `\addvspace` glue before this unit (list skips; paragraphs only).
    addvspace_before: f64,
    /// See [`Block::Paragraph::endlist_adjust`].
    endlist_adjust: f64,
    /// Constructs before this unit the pipeline set approximately.
    limitations: Vec<(&'static str, Span, String)>,
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
        /// A compiler `Styled` paragraph (`center`, `quote`, ...).
        styled: Option<ParaStyle>,
        /// The unit is the first paragraph of its environment (the gap
        /// before it holds `\begin{...}`); see [`EnvOpen`].
        env_open: Option<EnvOpen>,
        /// LaTeX's `\@endpe`: text that follows `\end{center}`/... without
        /// a blank line continues in the same paragraph, unindented.
        after_env: bool,
        /// A compiler `ListItem` paragraph: its `\list` geometry.
        list: Option<ListGeom>,
    },
    Rule {
        span: Span,
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

fn split_at_page_breaks<'p>(texts: &[&str], blocks: &'p [CBlock], size: u32, style: &Stylesheet) -> Vec<Unit<'p>> {
    let mut units = Vec::new();
    let mut prev_end: Option<Span> = None;
    // Carried from the compiler's own `PageBreak`/`VSpace`/`Rule` blocks
    // to the next unit that holds material.
    let mut pending_eject = false;
    let mut pending_vspace = 0.0f64;
    let mut pending_limitations: Vec<(&'static str, Span, String)> = Vec::new();
    // The previous unit left TeX in vertical mode (a heading or a rule).
    let mut prev_vmode = false;
    let mut prev_styled = false;
    // The previous unit was an `\item` paragraph, and whether its list's
    // `\begin` was read in vertical mode (`\@topsepadd` keeps `\partopsep`
    // for the closing skip too).
    let mut prev_list = false;
    let mut list_vmode = false;
    for block in blocks {
        match block {
            CBlock::PageBreak => {
                pending_eject = true;
                continue;
            }
            CBlock::VSpace { pt } => {
                pending_vspace += pt;
                continue;
            }
            CBlock::Rule { span } => {
                let eject = std::mem::take(&mut pending_eject) || prev_end.is_some_and(|p| gap_has_page_break(texts, p, *span));
                units.push(Unit {
                    kind: UnitKind::Rule { span: *span },
                    eject_before: eject,
                    vspace_before: std::mem::take(&mut pending_vspace),
                    addvspace_before: 0.0,
                    endlist_adjust: 0.0,
                    limitations: std::mem::take(&mut pending_limitations),
                });
                prev_end = Some(*span);
                prev_vmode = true;
                continue;
            }
            _ => {}
        }
        let first = match block {
            CBlock::Heading { number_span, .. } => Some(*number_span),
            _ => inlines_of(block).iter().map(inline_span).next(),
        };
        let mut eject = std::mem::take(&mut pending_eject) || matches!((prev_end, first), (Some(p), Some(f)) if gap_has_page_break(texts, p, f));
        let mut vspace_before = std::mem::take(&mut pending_vspace);
        // The compiler evaluates `em`/`ex` in `\vspace` at a fixed 12pt;
        // LaTeX uses the class's `\normalsize`. Re-read the commands in
        // the gap before this unit when they are all there.
        if vspace_before != 0.0 {
            if let Some(f) = first {
                let gap = match prev_end {
                    Some(p) if p.document == f.document && p.end <= f.start => texts.get(f.document.0).and_then(|t| t.get(p.end..f.start)),
                    Some(_) => None,
                    None => texts.get(f.document.0).and_then(|t| t.get(..f.start)),
                };
                if let Some(pt) = gap.and_then(|g| vspace_in_gap(g, size)) {
                    vspace_before = pt;
                }
            }
        }
        // The compiler's list model (pin `42557b09`): every `\item`
        // paragraph is a `ListItem` with its nesting level and, for the
        // item's first paragraph, the marker text. Its `\setlist`
        // itemsep/topsep gaps are attached to whichever paragraph the
        // *next* `\item`/`\end` flushes, so an item holding a display
        // (which ends the paragraph early) carries them on the wrong
        // block, and `em` in them is the compiler's fixed 12pt body; the
        // pipeline sets the list's vertical glue from the source instead:
        //
        // `\@item` of the first item: `\addvspace\@topsep` (`\topsep` +
        // the outer `\parskip`, + `\partopsep` when `\begin` was read in
        // vertical mode) then `\addvspace{-\parskip}` with `\parskip` now
        // `\parsep`; the item paragraph then adds `\parsep`
        // (`paragraph_block`). The first `\addvspace` only tops up the
        // skip the previous block left (a display's `\belowdisplayskip`),
        // while the negative one always takes `\parsep` off whatever is
        // there, so it goes into `vspace_before`. Right after a heading (`\@nobreak`)
        // `\@nbitem`'s skip is absorbed by the heading's after-skip, so
        // only `\parsep` remains. Later items: `\addvspace\itemsep`. A
        // later paragraph of one item (no label) adds nothing but
        // `\parsep`. `\end{...}`: `\@endparenv` adds `\@topsepadd`, absorbed
        // by a following heading's larger before-skip (`\addvspace`).
        // The hanging indent and the label box are the pipeline's too
        // (`list_margins`): the compiler reports `leftmargin` as
        // unimplemented.
        let gap_before = |f: Span| -> Option<&str> {
            match prev_end {
                Some(p) if p.document == f.document && p.end <= f.start => texts.get(f.document.0).and_then(|t| t.get(p.end..f.start)),
                Some(_) => None,
                None => texts.get(f.document.0).and_then(|t| t.get(..f.start)),
            }
        };
        let is_heading = matches!(block, CBlock::Heading { .. });
        let mut addvspace_before = 0.0;
        let mut endlist_adjust = 0.0;
        if prev_list && !is_heading {
            if let Some(gap) = first.and_then(gap_before) {
                if let Some(env) = gap_has_list_end(gap) {
                    let src = texts.get(prev_end.map_or(0, |p| p.document.0)).copied().unwrap_or("");
                    let seps = list_seps(src, env, 1, size, style);
                    addvspace_before += seps.topsep + if list_vmode { seps.partopsep } else { 0.0 };
                    if let Some(p) = prev_end {
                        endlist_adjust = list_end_adjust(src, p.end, gap, size, style);
                    }
                }
            }
        }
        let mut list = None;
        if let CBlock::ListItem { level, label, .. } = block {
            let anchor = label.as_ref().map(|(_, span)| *span).or(first);
            if let Some(at) = anchor {
                let src = texts.get(at.document.0).copied().unwrap_or("");
                let stack = list_stack_at(src, at.start);
                let env = stack.last().map_or("enumerate", |(env, _)| env);
                let seps = list_seps(src, env, stack.len().max(1), size, style);
                // `\@outerparskip`: the `\parskip` in force when `\begin`
                // was read — the enclosing list's `\parsep` when nested.
                let outer_parskip = match stack.len() {
                    n if n > 1 => list_seps(src, stack[n - 2].0, n - 1, size, style).parsep,
                    _ => style.parskip.natural,
                };
                if label.is_some() {
                    let opens = gap_before(at).and_then(|g| rfind_command(g, "begin").map(|b| (g, b)));
                    match opens {
                        Some((g, b)) if list_env_after_begin(&g[b..]) => {
                            let before = &g[..b];
                            list_vmode = prev_vmode || prev_end.is_none() || has_blank_line(before) || find_command(before, "par").is_some();
                            if prev_vmode {
                                // `\@nbitem`: `\addvspace{\@outerparskip - \parskip}`.
                                addvspace_before += outer_parskip - seps.parsep;
                            } else {
                                addvspace_before += seps.topsep + outer_parskip + if list_vmode { seps.partopsep } else { 0.0 };
                                vspace_before -= seps.parsep;
                            }
                        }
                        _ => addvspace_before += seps.itemsep,
                    }
                }
                list = Some(ListGeom {
                    level: *level,
                    margins: list_margins(src, at.start, size),
                    label: label.clone(),
                    parsep: seps.parsep_skip,
                });
            }
        }
        prev_list = list.is_some();
        let limitations = std::mem::take(&mut pending_limitations);
        let styled = match block {
            CBlock::Styled { style, .. } => Some(ParaStyle::of(*style)),
            _ => None,
        };
        // The environment opens here when the gap before the block holds
        // its `\begin`; `\partopsep` applies when that `\begin` was read in
        // vertical mode (nothing before it, or a blank line / `\par` between
        // the previous material and it).
        let env_open = styled.and_then(|_| {
            let f = first?;
            let gap = match prev_end {
                Some(p) if p.document == f.document && p.end <= f.start => texts.get(f.document.0).and_then(|t| t.get(p.end..f.start))?,
                Some(_) => return None,
                None => texts.get(f.document.0).and_then(|t| t.get(..f.start))?,
            };
            let begin = rfind_command(gap, "begin")?;
            let before = &gap[..begin];
            let vmode = prev_vmode || prev_end.is_none() || has_blank_line(before) || find_command(before, "par").is_some();
            Some(EnvOpen { vmode })
        });
        // `\@endpe`: a plain paragraph right after `\end{...}` (no blank line
        // or `\par` between them) is not indented.
        let after_env = styled.is_none()
            && prev_styled
            && first.zip(prev_end).is_some_and(|(f, p)| {
                p.document == f.document
                    && p.end <= f.start
                    && texts.get(f.document.0).and_then(|t| t.get(p.end..f.start)).is_some_and(|gap| {
                        rfind_command(gap, "end").is_some_and(|end| {
                            let after = gap[end..].split_once('}').map_or("", |(_, rest)| rest);
                            !has_blank_line(after) && find_command(after, "par").is_none()
                        })
                    })
            });
        prev_styled = styled.is_some();
        prev_vmode = matches!(block, CBlock::Heading { .. });
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
                    vspace_before,
                    addvspace_before,
                    endlist_adjust: 0.0,
                    limitations,
                });
            }
            CBlock::Paragraph(inlines) | CBlock::ListItem { content: inlines, .. } | CBlock::FigureCaption { content: inlines } | CBlock::Styled { content: inlines, .. } => {
                let caption = matches!(block, CBlock::FigureCaption { .. });
                let mut env_open = env_open;
                let mut start = 0usize;
                let mut vspace_before = vspace_before;
                let mut limitations = limitations;
                for i in 1..inlines.len() {
                    if gap_has_page_break(texts, inline_span(&inlines[i - 1]), inline_span(&inlines[i])) {
                        units.push(Unit {
                            kind: UnitKind::Paragraph {
                                inlines: &inlines[start..i],
                                caption,
                                styled,
                                env_open: env_open.take(),
                                after_env,
                                list: list.clone(),
                            },
                            eject_before: eject,
                            vspace_before: std::mem::take(&mut vspace_before),
                            addvspace_before: std::mem::take(&mut addvspace_before),
                            endlist_adjust: std::mem::take(&mut endlist_adjust),
                            limitations: std::mem::take(&mut limitations),
                        });
                        eject = true;
                        start = i;
                    }
                }
                units.push(Unit {
                    kind: UnitKind::Paragraph {
                        inlines: &inlines[start..],
                        caption,
                        styled,
                        env_open,
                        after_env,
                        list,
                    },
                    eject_before: eject,
                    vspace_before,
                    addvspace_before,
                    endlist_adjust,
                    limitations,
                });
            }
            CBlock::VSpace { .. } | CBlock::Rule { .. } | CBlock::PageBreak => unreachable!("handled above"),
            CBlock::Verbatim { .. } | CBlock::TableOfContents { .. } | CBlock::TitleBlock { .. } | CBlock::VFill => unreachable!("lowered by lower_blocks"),
        }
        if let Some(last) = inlines_of(block).iter().map(inline_span).last() {
            prev_end = Some(last);
        }
    }
    units
}

fn is_display(inlines: &[Inline], span: Span) -> bool {
    inlines.iter().any(|i| match i {
        Inline::Math { display: true, span: s, .. } => *s == span,
        Inline::MathRows { rows, .. } => rows.iter().any(|r| r.span == span),
        _ => false,
    })
}

/// The row of an amsmath multi-row display whose span is `span`, if any:
/// `Some(number)` where `number` is the row's own equation number.
fn math_row_number(inlines: &[Inline], span: Span) -> Option<Option<(String, Span)>> {
    inlines.iter().find_map(|i| match i {
        Inline::MathRows { rows, .. } => rows.iter().find(|r| r.span == span).map(|r| r.number.clone().map(|n| (n, r.span))),
        _ => None,
    })
}

/// One row of an amsmath display as a single math list: the `&`-separated
/// cells concatenated in order (the alignment points are reported by
/// `adapt` as a `math_limitation`).
fn math_row_list(row: &flashtex_compiler::parser::MathRow) -> MathList {
    MathList {
        atoms: row.cells.iter().flat_map(|c| c.atoms.iter().cloned()).collect(),
    }
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
    setlength(source, "parindent", size)
}

/// `\setlength{\parskip}{<dimen>}` in the source, in points (the compiler
/// reports the preamble command and drops it; LaTeX evaluates `em`/`ex`
/// in the class's `\normalsize`).
pub fn parskip(source: &str, size: u32) -> Option<f64> {
    setlength(source, "parskip", size)
}

/// The last `\setlength{\<name>}{<dimen>}` of the source, in points.
fn setlength(source: &str, name: &str, size: u32) -> Option<f64> {
    let needle = format!("{{\\{name}}}");
    let mut from = 0;
    let mut found = None;
    while let Some(at) = find_command(&source[from..], "setlength") {
        let abs = from + at;
        let rest = source[abs + "\\setlength".len()..].trim_start();
        if let Some(r) = rest.strip_prefix(needle.as_str()) {
            if let Some(r) = r.trim_start().strip_prefix('{') {
                if let Some(end) = r.find('}') {
                    found = parse_dimen(&r[..end], size).or(found);
                }
            }
        }
        from = abs + 1;
    }
    found
}

/// The class size (`10`/`11`/`12`) whose `\normalsize` is `body_pt`.
pub fn class_size_of(body_pt: f64) -> u32 {
    if body_pt >= 11.9 {
        12
    } else if body_pt >= 10.9 {
        11
    } else {
        10
    }
}

/// A TeX `<dimen>` in points; `em`/`ex` are those of the class's
/// `\normalsize` (`size` is the class size).
pub fn parse_dimen_pt(s: &str, size: u32) -> Option<f64> {
    parse_dimen(s, size)
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

/// The vertical glue of one list level, in points at the class size:
/// article's `\@list<i>` values (`document-style`), with the source's
/// `\setlist[<env>]{topsep=..,itemsep=..,parsep=..,partopsep=..}`
/// overrides for `env` (enumitem evaluates `em`/`ex` in `\normalsize`).
#[derive(Debug, Clone, Copy)]
struct ListSeps {
    topsep: f64,
    partopsep: f64,
    itemsep: f64,
    parsep: f64,
    /// `\parsep` with its stretch and shrink.
    parsep_skip: crate::style::Skip,
}

fn list_seps(source: &str, env: &str, depth: usize, size: u32, style: &Stylesheet) -> ListSeps {
    let base = match size {
        12 => flashtex_document_style::BaseSize::Pt12,
        11 => flashtex_document_style::BaseSize::Pt11,
        _ => flashtex_document_style::BaseSize::Pt10,
    };
    let class = flashtex_document_style::list_level(base, depth as u8);
    let mut seps = ListSeps {
        topsep: class.topsep.pt,
        partopsep: class.partopsep.pt,
        itemsep: class.itemsep.pt,
        parsep: class.parsep.pt,
        parsep_skip: crate::style::Skip::new(class.parsep.pt, class.parsep.plus, class.parsep.minus),
    };
    if depth == 1 {
        // The stylesheet's level-1 values are the ones the typesetter
        // reads for `\topsep`; keep both readings identical.
        seps.topsep = style.topsep.natural;
        seps.partopsep = style.partopsep.natural;
        seps.parsep = style.parsep.natural;
        seps.parsep_skip = style.parsep;
    }
    for (envs, keys) in setlist_calls(source) {
        if !setlist_names(envs, env) {
            continue;
        }
        for (key, value) in list_keys(keys) {
            let Some(pt) = parse_dimen(value, size) else { continue };
            match key {
                "topsep" => seps.topsep = pt,
                "partopsep" => seps.partopsep = pt,
                "itemsep" => seps.itemsep = pt,
                "parsep" => {
                    seps.parsep = pt;
                    seps.parsep_skip = crate::style::Skip::fixed(pt);
                }
                _ => {}
            }
        }
    }
    seps
}

/// Whether `rest` (starting at a `\begin`) opens `itemize`/`enumerate`.
fn list_env_after_begin(rest: &str) -> bool {
    let after = rest.strip_prefix("\\begin").unwrap_or(rest).trim_start();
    after.starts_with("{itemize}") || after.starts_with("{enumerate}")
}

/// `\endtrivlist` for every `\end{itemize}`/`\end{enumerate}` in `gap`
/// (which starts at byte `gap_start` of `source`), innermost first: when
/// the list leaves a positive `\lastskip` it becomes `\lastskip +
/// \parskip - \@outerparskip` — the closing list's `\parsep` less the
/// `\parskip` outside it (the enclosing list's `\parsep`, or the
/// document's). The summed change, in points.
fn list_end_adjust(source: &str, gap_start: usize, gap: &str, size: u32, style: &Stylesheet) -> f64 {
    let mut adjust = 0.0;
    let mut from = 0;
    while let Some(at) = find_command(&gap[from..], "end") {
        let abs = from + at;
        from = abs + 1;
        let rest = gap[abs + "\\end".len()..].trim_start();
        if !rest.starts_with("{itemize}") && !rest.starts_with("{enumerate}") {
            continue;
        }
        let stack = list_stack_at(source, gap_start + abs);
        let Some(&(env, _)) = stack.last() else { continue };
        let depth = stack.len();
        let parsep = list_seps(source, env, depth, size, style).parsep;
        let outer = if depth > 1 { list_seps(source, stack[depth - 2].0, depth - 1, size, style).parsep } else { style.parskip.natural };
        adjust += parsep - outer;
    }
    adjust
}

/// The environment of the last `\end{itemize}`/`\end{enumerate}` in `gap`.
fn gap_has_list_end(gap: &str) -> Option<&'static str> {
    let end = rfind_command(gap, "end")?;
    let rest = gap[end + "\\end".len()..].trim_start();
    ["itemize", "enumerate"].into_iter().find(|env| rest.strip_prefix('{').is_some_and(|r| r.starts_with(&format!("{env}}}"))))
}

/// The `\setlist[<envs>]{<keys>}` calls of `source`, in order:
/// `(environment list or "" for all, keys)`.
fn setlist_calls(source: &str) -> Vec<(&str, &str)> {
    let mut calls = Vec::new();
    let mut from = 0;
    while let Some(at) = find_command(&source[from..], "setlist") {
        let abs = from + at;
        from = abs + 1;
        let rest = &source[abs + "\\setlist".len()..];
        let rest = rest.strip_prefix('*').unwrap_or(rest).trim_start();
        let (envs, rest) = match rest.strip_prefix('[') {
            Some(r) => match r.find(']') {
                Some(close) => (&r[..close], r[close + 1..].trim_start()),
                None => continue,
            },
            None => ("", rest),
        };
        if !rest.starts_with('{') {
            continue;
        }
        let Some(close) = matching_brace(rest.as_bytes(), 0) else { continue };
        calls.push((envs, &rest[1..close]));
    }
    calls
}

/// enumitem `key=value` pairs (a key without `=` gets an empty value).
fn list_keys(keys: &str) -> impl Iterator<Item = (&str, &str)> {
    keys.split(',').map(str::trim).filter(|k| !k.is_empty()).map(|k| match k.split_once('=') {
        Some((key, value)) => (key.trim(), value.trim()),
        None => (k, ""),
    })
}

/// Whether a `\setlist[<envs>]` list names `env` (enumitem also accepts
/// level numbers there, which apply to every environment).
fn setlist_names(envs: &str, env: &str) -> bool {
    envs.trim().is_empty() || envs.split(',').map(str::trim).any(|e| e == env || e.parse::<u8>().is_ok())
}

/// The `itemize`/`enumerate` environments open at byte `at` of `source`,
/// outermost first: `(environment, `\begin` optional argument)`.
fn list_stack_at(source: &str, at: usize) -> Vec<(&str, &str)> {
    let mut stack: Vec<(&str, &str)> = Vec::new();
    let mut from = 0;
    while from < at {
        let next_begin = find_command(&source[from..at], "begin").map(|i| from + i);
        let next_end = find_command(&source[from..at], "end").map(|i| from + i);
        let (pos, is_begin) = match (next_begin, next_end) {
            (Some(b), Some(e)) if b < e => (b, true),
            (Some(b), None) => (b, true),
            (_, Some(e)) => (e, false),
            (None, None) => break,
        };
        from = pos + 1;
        let rest = &source[pos + if is_begin { "\\begin".len() } else { "\\end".len() }..];
        let rest = rest.trim_start();
        let Some(inner) = rest.strip_prefix('{') else { continue };
        let Some(close) = inner.find('}') else { continue };
        let env = inner[..close].trim();
        if !matches!(env, "itemize" | "enumerate") {
            continue;
        }
        if is_begin {
            let after = inner[close + 1..].trim_start();
            let options = match after.strip_prefix('[') {
                Some(o) => o.find(']').map_or("", |c| &o[..c]),
                None => "",
            };
            stack.push((env, options));
        } else if stack.last().is_some_and(|(open, _)| *open == env) {
            stack.pop();
        }
    }
    stack
}

/// article's `\leftmargin<i>` for nesting `depth` (1-based), in em of
/// the body font (`\leftmarginv`/`vi` are 1em).
fn article_leftmargin_em(depth: usize) -> f64 {
    [2.5, 2.2, 1.87, 1.7, 1.0, 1.0][depth.clamp(1, 6) - 1]
}

/// The widest label enumitem assumes for the `leftmargin=*` computation:
/// a `label=` key (its `\alph*`-style counter replaced by `m`/`M`/
/// `viii`/`VIII`/`0`), a shortlabels template (`(a)` -> `(m)`), or the
/// class's own label for this depth.
fn widest_label(env: &str, depth: usize, label_key: Option<&str>, template: Option<&str>) -> String {
    if let Some(label) = label_key {
        return [("\\alph*", "m"), ("\\Alph*", "M"), ("\\roman*", "viii"), ("\\Roman*", "VIII"), ("\\arabic*", "0")]
            .iter()
            .fold(label.to_string(), |text, (command, widest)| text.replace(command, widest));
    }
    if env == "itemize" {
        return match depth {
            1 => "•",
            2 => "–",
            3 => "∗",
            _ => "·",
        }
        .to_string();
    }
    if let Some(template) = template {
        if let Some((index, style)) = template.char_indices().find(|(_, c)| "aAiI1".contains(*c)) {
            let widest = match style {
                'a' => "m",
                'A' => "M",
                'i' => "viii",
                'I' => "VIII",
                _ => "0",
            };
            return format!("{}{}{}", &template[..index], widest, &template[index + 1..]);
        }
        return template.to_string();
    }
    match depth {
        1 => "0.",
        2 => "(m)",
        3 => "viii.",
        _ => "M.",
    }
    .to_string()
}

/// `\leftmargin` of every list open at byte `at` (outermost first): the
/// class's `\leftmargin<i>` unless a `\setlist` naming the environment or
/// the `\begin` options set enumitem's `leftmargin` (`*` = the widest
/// label's width plus `\labelsep`; a `<dimen>` as given).
fn list_margins(source: &str, at: usize, size: u32) -> Vec<ListMargin> {
    let calls = setlist_calls(source);
    let class_margin = |depth: usize| ListMargin::Fixed(parse_dimen(&format!("{}em", article_leftmargin_em(depth)), size).unwrap_or(0.0));
    list_stack_at(source, at)
        .iter()
        .enumerate()
        .map(|(i, (env, options))| {
            let depth = i + 1;
            let mut leftmargin: Option<&str> = None;
            let mut label_key: Option<&str> = None;
            let begin_keys = options.contains('=');
            let all_keys = calls
                .iter()
                .filter(|(envs, _)| setlist_names(envs, env))
                .map(|(_, keys)| *keys)
                .chain(begin_keys.then_some(*options));
            for keys in all_keys {
                for (key, value) in list_keys(keys) {
                    match key {
                        "leftmargin" => leftmargin = Some(value),
                        "label" => label_key = Some(value),
                        _ => {}
                    }
                }
            }
            let template = (!begin_keys && !options.is_empty()).then_some(*options);
            match leftmargin {
                Some("*") => ListMargin::Widest(widest_label(env, depth, label_key, template)),
                Some(dimen) => parse_dimen(dimen, size).map_or_else(|| class_margin(depth), ListMargin::Fixed),
                None => class_margin(depth),
            }
        })
        .collect()
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

/// Byte offset of the last `\<name>` in `source` outside comments.
fn rfind_command(source: &str, name: &str) -> Option<usize> {
    let mut last = None;
    let mut from = 0;
    while let Some(at) = find_command(&source[from..], name) {
        last = Some(from + at);
        from += at + 1;
    }
    last
}

/// Whether `source` holds a blank line (TeX's `\par` from an empty line):
/// two newlines with only blanks between them.
fn has_blank_line(source: &str) -> bool {
    let mut newlines = 0;
    for c in source.chars() {
        match c {
            '\n' => {
                newlines += 1;
                if newlines >= 2 {
                    return true;
                }
            }
            ' ' | '\t' | '\r' => {}
            _ => newlines = 0,
        }
    }
    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StyleKind {
    Bold,
    Emph,
    Italic,
}

/// The point size a `\tiny`..`\Huge` declaration selects at a class base
/// size (size10/11/12.clo), in hundredths of a point; 0 for `\normalsize`
/// (the paragraph's own size). The declaration in force comes from the
/// compiler's `TextStyle::size` (pin `b38e1884`, declaration-scoped like
/// bold/italic); the source scan below no longer reads size declarations,
/// so a size is never applied twice. The compiler's own table is the same
/// one, but it resolves against its integer class size where the pipeline
/// sets `\normalsize` at the class's real `\normalsize` (10.95pt at 11pt).
fn declared_size(level: Option<flashtex_compiler::parser::FontSizeLevel>, base: u32) -> u16 {
    use flashtex_compiler::parser::FontSizeLevel as L;
    let Some(level) = level else { return 0 };
    // tiny, scriptsize, footnotesize, small, large, Large, LARGE, huge, Huge
    let table: [[u16; 3]; 9] = [
        [500, 600, 600],
        [700, 800, 800],
        [800, 900, 1000],
        [900, 1000, 1095],
        [1200, 1200, 1440],
        [1440, 1440, 1728],
        [1728, 1728, 2074],
        [2074, 2074, 2488],
        [2488, 2488, 2488],
    ];
    let col = match base {
        11 => 1,
        12 => 2,
        _ => 0,
    };
    let row = match level {
        L::Tiny => 0,
        L::ScriptSize => 1,
        L::FootnoteSize => 2,
        L::Small => 3,
        L::Large1 => 4,
        L::Large2 => 5,
        L::Large3 => 6,
        L::Huge1 => 7,
        L::Huge2 => 8,
    };
    table[row][col]
}

/// Brace-group intervals of `\textbf{}`, `\emph{}`, `\textit{}` in source
/// byte offsets (content only), in document order. Weight and shape only:
/// the compiler carries no `\bfseries`/`\itshape` scoping for body text
/// the pipeline could use, while size declarations are read from the
/// compiler's `TextStyle::size` (see [`declared_size`]).
fn style_intervals(source: &str) -> Vec<(usize, usize, StyleKind)> {
    let mut out = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0;
    let mut in_comment = false;
    // Open brace groups (byte of `{`): a declaration (`\bfseries`,
    // `\Large`, ...) lasts to the end of the innermost one, or to the next
    // `\end{...}`/the document end outside any group.
    let mut groups: Vec<usize> = Vec::new();
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
            b'{' => {
                groups.push(i);
                i += 1;
            }
            b'}' => {
                groups.pop();
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
                        continue;
                    }
                    None => {}
                }
                // Declarations: the control word's letters.
                let word_end = i + 1 + rest[1..].bytes().take_while(u8::is_ascii_alphabetic).count();
                let name = &source[i + 1..word_end];
                let decl = match name {
                    "bfseries" => Some(StyleKind::Bold),
                    "itshape" | "slshape" => Some(StyleKind::Italic),
                    "em" => Some(StyleKind::Emph),
                    _ => None,
                };
                if let Some(k) = decl {
                    let end = match groups.last() {
                        Some(&open) => matching_brace(bytes, open).unwrap_or(bytes.len()),
                        None => find_command(&source[word_end..], "end").map_or(bytes.len(), |e| word_end + e),
                    };
                    out.push((word_end, end, k));
                }
                i = word_end.max(i + 2);
            }
            _ => i += 1,
        }
    }
    out.sort_by_key(|(start, _, _)| *start);
    out
}

fn continues_word(bytes: &[u8], at: usize) -> bool {
    at < bytes.len() && bytes[at].is_ascii_alphabetic()
}

/// The control word (`\name`, letters only) that starts at byte `at`, if
/// the source holds one there and it ends before `end`.
fn control_word_at(source: &str, at: usize, end: usize) -> Option<&str> {
    let rest = source.get(at..end)?;
    let rest = rest.strip_prefix('\\')?;
    let len = rest.bytes().take_while(u8::is_ascii_alphabetic).count();
    if len == 0 || len != rest.len() {
        return None;
    }
    Some(&rest[..len])
}

/// Whether the bytes at `at` are exactly the control word `\<name>` (not
/// a longer word: `\hfil` is not `\hfill`).
fn is_control_word(source: &str, at: usize, name: &str) -> bool {
    let bytes = source.as_bytes();
    source.get(at..).is_some_and(|r| r.starts_with('\\') && r[1..].starts_with(name)) && !continues_word(bytes, at + 1 + name.len())
}

/// Whether `span` is a user-macro invocation (`\name` exactly, with a
/// `\newcommand`-style definition in the source): the compiler gives every
/// token of the replacement text this span.
fn is_invocation_span(source: &str, span: Span) -> bool {
    control_word_at(source, span.start, span.end).is_some_and(|name| macro_body(source, name, span.start).is_some())
}

/// The replacement text of the last `\newcommand`/`\renewcommand`/
/// `\providecommand`/`\def` for `\<name>` before byte `before` (or the first
/// one anywhere), as the bytes inside its braces.
fn macro_body<'a>(source: &'a str, name: &str, before: usize) -> Option<&'a str> {
    let bytes = source.as_bytes();
    let mut defs: Vec<(usize, &str)> = Vec::new();
    for command in ["newcommand", "renewcommand", "providecommand", "def"] {
        let mut from = 0;
        while let Some(at) = find_command(&source[from..], command) {
            let abs = from + at;
            from = abs + 1;
            let mut i = abs + 1 + command.len();
            let skip_ws = |i: &mut usize| {
                while *i < bytes.len() && (bytes[*i] as char).is_whitespace() {
                    *i += 1;
                }
            };
            skip_ws(&mut i);
            if bytes.get(i) == Some(&b'*') {
                i += 1;
                skip_ws(&mut i);
            }
            // `{\name}` or `\name`.
            let braced = bytes.get(i) == Some(&b'{');
            if braced {
                i += 1;
                skip_ws(&mut i);
            }
            let Some(rest) = source.get(i..) else { continue };
            let Some(rest) = rest.strip_prefix('\\') else { continue };
            let len = rest.bytes().take_while(u8::is_ascii_alphabetic).count();
            if &rest[..len] != name {
                continue;
            }
            i += 1 + len;
            skip_ws(&mut i);
            if braced {
                if bytes.get(i) != Some(&b'}') {
                    continue;
                }
                i += 1;
            }
            // `[n]`, `[default]` (LaTeX) or `#1#2` (`\def`).
            loop {
                skip_ws(&mut i);
                match bytes.get(i) {
                    Some(b'[') => match source[i..].find(']') {
                        Some(close) => i += close + 1,
                        None => break,
                    },
                    Some(b'#') => i += 2,
                    _ => break,
                }
            }
            if bytes.get(i) != Some(&b'{') {
                continue;
            }
            let Some(close) = matching_brace(bytes, i) else { continue };
            defs.push((abs, &source[i + 1..close]));
        }
    }
    defs.sort_by_key(|(at, _)| *at);
    defs.iter().rev().find(|(at, _)| *at < before).or(defs.first()).map(|(_, body)| *body)
}

/// For a macro invoked at `inv` (its `\name` span), the index (from 1) of
/// the brace-delimited argument whose bytes contain `at`, and the macro's
/// name.
fn macro_arg_index(source: &str, inv: Span, at: usize) -> Option<(&str, usize)> {
    let name = control_word_at(source, inv.start, inv.end)?;
    let bytes = source.as_bytes();
    let mut i = inv.end;
    let mut k = 0usize;
    loop {
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        match bytes.get(i) {
            Some(b'[') => {
                let close = source[i..].find(']')?;
                i += close + 1;
            }
            Some(b'{') => {
                let close = matching_brace(bytes, i)?;
                k += 1;
                if at > i && at < close {
                    return Some((name, k));
                }
                i = close + 1;
            }
            _ => return None,
        }
    }
}

/// Where the reader stands inside a macro's replacement text: the
/// invocation (`\name` span the compiler gives every replacement token)
/// and the byte offset in its definition body after the last token read.
#[derive(Debug, Clone, Copy)]
struct BodyCursor {
    inv: Span,
    at: usize,
}

/// The bytes TeX read between the previous token and this one, in the
/// order it read them, so that [`gap_has_space`] can be
/// applied to them uniformly: the source between two exact spans; the
/// definition text between two tokens of one replacement (`\hfill
/// \normalfont[` between `#1` and `[`); and across the boundary between a
/// replacement token and an argument (the whitespace around `#k`). `text`
/// is the token's text, used to find its place in a definition. `None`
/// when nothing was read (the first token) or the place is unknown.
fn token_gap(src: &str, prev_end: Option<usize>, prev_span: Option<Span>, span: Span, text: Option<&str>, cursor: &mut Option<BodyCursor>) -> Option<String> {
    let source_gap = |pe: usize, ps: Span| -> Option<String> {
        if ps.document != span.document {
            // Crossing an \input boundary: TeX reads the newline that ends
            // the \input line as a space.
            Some(" ".to_string())
        } else {
            (pe <= span.start).then(|| src.get(pe..span.start).unwrap_or("").to_string())
        }
    };
    let digits = |k: usize| 1 + k.to_string().len();
    // A word of a replacement text.
    if let Some(name) = control_word_at(src, span.start, span.end) {
        if let Some(body) = macro_body(src, name, span.start) {
            let (start, prefix) = match *cursor {
                Some(c) if c.inv == span => (c.at, None),
                _ => match prev_span.and_then(|ps| macro_arg_index(src, span, ps.start)) {
                    // The previous token was an argument of this invocation.
                    Some((_, k)) => (body.find(&format!("#{k}")).map_or(0, |p| p + digits(k)), None),
                    None => (0, prev_end.zip(prev_span).map(|(pe, ps)| source_gap(pe, ps).unwrap_or_default())),
                },
            };
            let Some(text) = text else {
                // Glue or math of a replacement: its place is not searched;
                // separate tokens of one replacement are taken as spaced.
                *cursor = Some(BodyCursor { inv: span, at: start });
                return prev_end.map(|_| " ".to_string());
            };
            // A control word (the glue arms pass `\hfill`/`\quad`/...) is
            // matched as a whole word, so `\hfil` never stops at `\hfill`.
            let find_text = |rest: &str| match text.strip_prefix('\\') {
                Some(name) if name.chars().all(|c| c.is_ascii_alphabetic()) => find_command(rest, name),
                _ => rest.find(text),
            };
            match body.get(start..).and_then(find_text) {
                Some(p) => {
                    let pos = start + p;
                    *cursor = Some(BodyCursor { inv: span, at: pos + text.len() });
                    let gap = &body[start..pos];
                    return Some(match prefix {
                        Some(before) => format!("{before}{gap}"),
                        None => gap.to_string(),
                    });
                }
                None => {
                    // Not found verbatim (ligatures rewrote it).
                    *cursor = Some(BodyCursor { inv: span, at: start });
                    return prev_end.map(|_| " ".to_string());
                }
            }
        }
    }
    // An argument of the invocation being read.
    if let Some(c) = *cursor {
        if let Some((name, k)) = macro_arg_index(src, c.inv, span.start) {
            if let Some(body) = macro_body(src, name, c.inv.start) {
                if let Some(p) = body.get(c.at..).and_then(|rest| rest.find(&format!("#{k}"))) {
                    let pos = c.at + p;
                    let gap = body[c.at..pos].to_string();
                    *cursor = Some(BodyCursor { inv: c.inv, at: pos + digits(k) });
                    return Some(gap);
                }
                // Further tokens of the same argument: the source between
                // them (the cursor stays after `#k`).
                return prev_end.zip(prev_span).and_then(|(pe, ps)| source_gap(pe, ps));
            }
        }
    }
    *cursor = None;
    prev_end.zip(prev_span).and_then(|(pe, ps)| source_gap(pe, ps))
}

/// [`gap_has_space`] for the bytes after a control word: the whitespace
/// TeX eats right after the word does not count.
fn gap_has_space_after_control_word(rest: &str) -> bool {
    let rest = rest.trim_start_matches([' ', '\t']);
    // A newline right after the word is eaten too (it is the same skip).
    let rest = rest.strip_prefix('\n').unwrap_or(rest);
    gap_has_space(rest)
}

/// The sum of every `\vspace{<dimen>}`/`\vspace*{<dimen>}` in `gap`, in
/// points; `None` when there is none or one does not parse.
fn vspace_in_gap(gap: &str, size: u32) -> Option<f64> {
    let mut from = 0;
    let mut total = 0.0;
    let mut any = false;
    while let Some(at) = find_command(&gap[from..], "vspace") {
        let abs = from + at;
        from = abs + 1;
        let rest = gap[abs + "\\vspace".len()..].trim_start();
        let rest = rest.strip_prefix('*').unwrap_or(rest).trim_start();
        let inner = rest.strip_prefix('{')?;
        let close = inner.find('}')?;
        total += parse_dimen(&inner[..close], size)?;
        any = true;
    }
    any.then_some(total)
}

/// The `[<dimen>]` of `\\[<dimen>]`/`\\*[<dimen>]` whose `\\` ends at byte
/// `after`, in points.
fn line_break_skip(source: &str, after: usize, size: u32) -> Option<f64> {
    let rest = source.get(after..)?;
    let rest = rest.strip_prefix('*').unwrap_or(rest);
    let rest = rest.trim_start_matches([' ', '\t']);
    let inner = rest.strip_prefix('[')?;
    let close = inner.find(']')?;
    parse_dimen(&inner[..close], size)
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
pub(crate) fn space_factor(ch: char, previous: u32) -> u32 {
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

/// [`items_from_inlines`] through the cross-request cache. The key covers
/// the inlines (kinds, texts, relative spans, label/reference keys), the
/// source bytes they sit in (gaps decide spaces, groups decide styles and
/// italic corrections), the style in force at the start, and the label
/// table; the value is relocated by the block's byte offset.
#[allow(clippy::too_many_arguments)]
fn items_cached(
    texts: &[&str],
    inlines: &[Inline],
    styles: &[Styles],
    labels: &Labels,
    labels_fp: u64,
    size: u32,
    heading: bool,
    cache: Option<&crate::incremental::RenderCache>,
) -> Vec<Item> {
    let Some(cache) = cache else {
        return items_from_inlines(texts, inlines, styles, labels, size, heading);
    };
    let Some(first) = inlines.first().map(inline_span) else {
        return items_from_inlines(texts, inlines, styles, labels, size, heading);
    };
    let document = first.document;
    let mut start = first.start;
    let mut end = first.end;
    for i in inlines {
        let s = inline_span(i);
        if s.document != document {
            return items_from_inlines(texts, inlines, styles, labels, size, heading);
        }
        start = start.min(s.start);
        end = end.max(s.end);
    }
    let Some(src) = texts.get(document.0) else {
        return items_from_inlines(texts, inlines, styles, labels, size, heading);
    };
    // Macro replacement text carries the invocation's span: the spacing
    // and weight of its words come from the definition (`macro_body`), so
    // a block holding one cannot be keyed by its own bytes alone.
    if inlines.iter().any(|i| is_invocation_span(src, inline_span(i))) {
        return items_from_inlines(texts, inlines, styles, labels, size, heading);
    }
    // `\\[<dimen>]` reads past the block's last span: the key covers the
    // rest of that line.
    let slice_end = src[end.min(src.len())..].find('\n').map_or(src.len(), |n| end + n + 1).max((end + 2).min(src.len()));
    let Some(slice) = src.get(start..slice_end) else {
        return items_from_inlines(texts, inlines, styles, labels, size, heading);
    };
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    b'A'.hash(&mut h);
    document.0.hash(&mut h);
    slice.hash(&mut h);
    labels_fp.hash(&mut h);
    size.hash(&mut h);
    heading.hash(&mut h);
    let no_styles = Styles::default();
    let st = styles.get(document.0).unwrap_or(&no_styles);
    let at = st.at(start);
    at.bold.hash(&mut h);
    at.italic.hash(&mut h);
    for i in inlines {
        let s = inline_span(i);
        (s.start.wrapping_sub(start), s.end.wrapping_sub(start)).hash(&mut h);
        match i {
            Inline::Text { text, .. } => {
                0u8.hash(&mut h);
                text.hash(&mut h);
            }
            Inline::LineBreak { .. } => 1u8.hash(&mut h),
            Inline::Math { list, display, number, .. } => {
                2u8.hash(&mut h);
                display.hash(&mut h);
                number.hash(&mut h);
                crate::incremental::hash_math(list, &mut h);
            }
            Inline::Label { key, value, .. } => {
                3u8.hash(&mut h);
                key.hash(&mut h);
                value.hash(&mut h);
            }
            Inline::Reference { key, page, equation, .. } => {
                4u8.hash(&mut h);
                key.hash(&mut h);
                page.hash(&mut h);
                equation.hash(&mut h);
            }
            // Lowered constructs (pin `d416472a`): their text is in the
            // source slice already hashed; the structure is hashed here.
            Inline::Footnote { number, mark, text, .. } => {
                9u8.hash(&mut h);
                number.hash(&mut h);
                mark.hash(&mut h);
                text.as_ref().map_or(0, Vec::len).hash(&mut h);
            }
            Inline::Tabular(t) => {
                10u8.hash(&mut h);
                t.entries.len().hash(&mut h);
                t.inline_lists().iter().map(|l| l.len()).sum::<usize>().hash(&mut h);
            }
            Inline::Verbatim { text, .. } => {
                11u8.hash(&mut h);
                text.hash(&mut h);
            }
            Inline::HFill { .. } => 6u8.hash(&mut h),
            Inline::HSpace { pt, .. } => {
                7u8.hash(&mut h);
                pt.to_bits().hash(&mut h);
            }
            Inline::TextGlue { em, .. } => {
                8u8.hash(&mut h);
                em.to_bits().hash(&mut h);
            }
            Inline::MathRows { rows, aligned, .. } => {
                5u8.hash(&mut h);
                aligned.hash(&mut h);
                rows.len().hash(&mut h);
                for row in rows {
                    (row.span.start.wrapping_sub(start), row.span.end.wrapping_sub(start)).hash(&mut h);
                    row.number.hash(&mut h);
                    row.cells.len().hash(&mut h);
                    for cell in &row.cells {
                        crate::incremental::hash_math(cell, &mut h);
                    }
                }
            }
        }
    }
    let key = h.finish();
    if let Some(a) = cache.adapted(key) {
        return crate::incremental::relocate_items(&a.items, start as isize - a.base as isize);
    }
    let items = items_from_inlines(texts, inlines, styles, labels, size, heading);
    cache.insert_adapted(key, crate::incremental::AdaptedBlock { items: items.clone(), base: start });
    items
}

/// Converts the compiler inlines into words, spaces, math and line breaks.
/// `texts` and `styles` are indexed by `DocumentId`; `size` is the class
/// size (for `em` in `\\[<dimen>]`); `heading` marks `\section{...}`
/// content, whose compiler styles start bold (`\normalfont` in it is read
/// from the compiler's own style, since the macro-expanded bytes are not in
/// the source at the invocation).
fn items_from_inlines(texts: &[&str], inlines: &[Inline], styles: &[Styles], labels: &Labels, size: u32, heading: bool) -> Vec<Item> {
    // `\ref`/`\pageref` become ordinary text attributed to the command's
    // bytes; `\label` becomes a zero-width marker.
    let mut resolved: Vec<std::borrow::Cow<Inline>> = Vec::with_capacity(inlines.len());
    let mut reference_spans: Vec<Span> = Vec::new();
    for inline in inlines {
        lower_inline(inline, labels, &mut reference_spans, &mut resolved);
    }
    let mut items: Vec<Item> = Vec::new();
    let mut prev_end: Option<usize> = None;
    let mut prev_span: Option<Span> = None;
    let mut factor = 1000u32;
    let mut pending_accent: Option<(char, CharSrc)> = None;
    // The compiler's size declaration in force at the previous text
    // inline, for the interword space read after it.
    let mut prev_size_cpt = 0u16;
    let text_of = |d: DocumentId| -> &str { texts.get(d.0).copied().unwrap_or("") };
    let no_styles = Styles::default();
    let styles_of = |d: DocumentId| -> &Styles { styles.get(d.0).unwrap_or(&no_styles) };

    // Where the reader stands in a macro's replacement text (`token_gap`).
    let mut cursor: Option<BodyCursor> = None;
    // Whether the previous token was a glue control word (`\hfill`,
    // `\quad`, `\hspace`): TeX eats the whitespace right after it, and
    // the gap read next starts at that whitespace.
    let mut after_control_word = false;
    // Whether the bytes TeX read between `prev` and `span` held an
    // interword space (`token_gap`). `text` is the current token's text (a
    // word, or the glue's control word). `\hfill` in a title is the
    // compiler's own `Inline::HFill` (pin `3d3d5ae3`, also inside macro
    // bodies), so the gap is never scanned for fills here.
    let mut space_between = |prev_end: Option<usize>, prev_span: Option<Span>, span: Span, text: Option<&str>, after_control_word: bool| -> bool {
        let src = text_of(span.document);
        match token_gap(src, prev_end, prev_span, span, text, &mut cursor) {
            None => false,
            Some(gap) if after_control_word => gap_has_space_after_control_word(&gap),
            Some(gap) => gap_has_space(&gap),
        }
    };
    // Pushes the space `space_between` found.
    let push_gap = |items: &mut Vec<Item>, space: bool, style: TextStyle, factor: u32| {
        if space {
            items.push(Item::Space { style, factor, no_break: false });
        }
    };

    for inline in resolved.iter() {
        match &**inline {
            Inline::Label { key, .. } => items.push(Item::Label { key: key.clone() }),
            Inline::Reference { .. } | Inline::Footnote { .. } | Inline::Tabular(_) | Inline::Verbatim { .. } => unreachable!("lowered by lower_inline above"),
            Inline::LineBreak { span } => {
                let skip_pt = line_break_skip(text_of(span.document), span.end, size).unwrap_or(0.0);
                items.push(Item::LineBreak { skip_pt });
                prev_end = Some(span.end);
                prev_span = Some(*span);
                factor = 1000;
                after_control_word = false;
            }
            Inline::HFill { span } | Inline::HSpace { span, .. } | Inline::TextGlue { span, .. } => {
                // Explicit horizontal glue: the interword space read before
                // it stays (TeX keeps both glue nodes). `TextGlue` is the
                // compiler's text-mode `\quad`/`\qquad` (`em` ems of the
                // current font, like the `\quad` after a section number).
                // The control word is passed as the token's text so that
                // a macro-body cursor moves past it (`Problem #1 \hfill
                // \normalfont[#2 points]`): the compiler gives the glue the
                // invocation's span, and the next token's gap must start
                // after the word, not before it.
                let (item, word) = match &**inline {
                    Inline::HSpace { pt, .. } => (Item::HSpace { pt: *pt }, "\\hspace"),
                    Inline::TextGlue { em, .. } => (Item::Quad { em: *em }, if *em >= 2.0 { "\\qquad" } else { "\\quad" }),
                    _ => {
                        let fill = !is_control_word(text_of(span.document), span.start, "hfil");
                        (Item::HFill { fill }, if fill { "\\hfill" } else { "\\hfil" })
                    }
                };
                let gap = space_between(prev_end, prev_span, *span, Some(word), after_control_word);
                let mut gap_style = space_style(texts, styles, prev_end, *span, TextStyle::default());
                gap_style.size_cpt = space_size(texts, prev_end, *span, prev_size_cpt, 0);
                push_gap(&mut items, gap, gap_style, factor);
                items.push(item);
                prev_end = Some(span.end);
                prev_span = Some(*span);
                factor = 1000;
                pending_accent = None;
                after_control_word = true;
            }
            Inline::MathRows { rows, span, .. } => {
                // Each row becomes its own display item (`is_display`
                // recognises the row spans); the environment's span ends
                // the preceding text like `\[`.
                let gap = space_between(prev_end, prev_span, *span, None, after_control_word);
                let mut gap_style = space_style(texts, styles, prev_end, *span, TextStyle::default());
                gap_style.size_cpt = space_size(texts, prev_end, *span, prev_size_cpt, 0);
                push_gap(&mut items, gap, gap_style, factor);
                after_control_word = false;
                for row in rows {
                    items.push(Item::Math {
                        list: math_row_list(row),
                        span: row.span,
                    });
                }
                prev_end = Some(span.end);
                prev_span = Some(*span);
                factor = 1000;
            }
            Inline::Math { list, span, .. } => {
                // The glue is the current font's where the space sits.
                let gap = space_between(prev_end, prev_span, *span, None, after_control_word);
                let mut gap_style = space_style(texts, styles, prev_end, *span, TextStyle::default());
                gap_style.size_cpt = space_size(texts, prev_end, *span, prev_size_cpt, 0);
                push_gap(&mut items, gap, gap_style, factor);
                after_control_word = false;
                items.push(Item::Math {
                    list: list.clone(),
                    span: *span,
                });
                prev_end = Some(span.end);
                prev_span = Some(*span);
                factor = 1000;
            }
            Inline::Text { text, span, .. } => {
                let source = text_of(span.document);
                // The compiler (pin `8c0d65e7`) runs its text-ligature pass
                // over the accent command's own character too, so `\'` and
                // `\`` arrive as the curly quotes; map them back.
                let accent_mark = |t: &str| -> Option<char> {
                    let mut it = t.chars();
                    match (it.next(), it.next()) {
                        (Some('\u{2019}'), None) => Some('\''),
                        (Some('\u{2018}'), None) => Some('`'),
                        (Some(c), None) if "\"'`^~=.".contains(c) => Some(c),
                        _ => None,
                    }
                };
                let accent_char = if span.end - span.start == 2 && source.as_bytes().get(span.start) == Some(&b'\\') {
                    accent_mark(text)
                } else {
                    None
                };
                let mut style = style_at(styles_of(span.document), span.start);
                // `\tiny`..`\Huge` come from the compiler's scoping.
                let Inline::Text { style: compiler_style, .. } = &**inline else { unreachable!() };
                style.size_cpt = declared_size(compiler_style.size, size);
                if heading {
                    // `\@startsection` sets `\bfseries`; the compiler's
                    // heading styles start bold and `\normalfont`/
                    // `\mdseries` in the title clears it.
                    let Inline::Text { style: cs, .. } = &**inline else { unreachable!() };
                    style.medium = !cs.bold;
                    style.italic |= cs.italic;
                }
                let has_space = space_between(prev_end, prev_span, *span, Some(text), after_control_word);
                after_control_word = false;
                if has_space {
                    // TeX sizes an interword space with the font current
                    // where the space token is read ("Plain, \textbf{bold}"
                    // gets a regular space, "\textbf{bold words}" a bold one,
                    // "\textbf{\emph{x}} y" a regular one).
                    let mut gap_style = space_style(texts, styles, prev_end, *span, style);
                    gap_style.size_cpt = space_size(texts, prev_end, *span, prev_size_cpt, style.size_cpt);
                    push_gap(&mut items, has_space, gap_style, factor);
                    pending_accent = None;
                }
                prev_size_cpt = style.size_cpt;
                if let Some(mark) = accent_char {
                    pending_accent = Some((
                        mark,
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

/// The size declaration in force where TeX reads the space token between
/// the previous inline (ending at `prev_end`) and `span`, from the
/// compiler's sizes of the two neighbours: the previous inline's unless a
/// closing brace precedes the gap's first whitespace (`{\Large x} y`: the
/// group has ended, so the space is read at the next inline's size).
fn space_size(texts: &[&str], prev_end: Option<usize>, span: Span, prev_cpt: u16, next_cpt: u16) -> u16 {
    if prev_cpt == next_cpt {
        return prev_cpt;
    }
    let Some(pe) = prev_end else { return next_cpt };
    let Some(gap) = texts.get(span.document.0).and_then(|t| t.get(pe..span.start)) else { return next_cpt };
    let ws = gap.find(|c: char| c.is_whitespace()).unwrap_or(gap.len());
    if gap[..ws].contains('}') {
        next_cpt
    } else {
        prev_cpt
    }
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
            Block::Rule { .. } => panic!("a rule holds no items"),
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

    /// Shorthand for an item list: `W` word, `S` space, `F` fill, `Q` quad.
    fn shape(items: &[Item]) -> String {
        items
            .iter()
            .map(|i| match i {
                Item::Word(_) => 'W',
                Item::Space { .. } => 'S',
                Item::HFill { .. } => 'F',
                Item::Quad { .. } => 'Q',
                Item::HSpace { .. } => 'H',
                _ => '?',
            })
            .collect()
    }

    #[test]
    fn compiler_glue_is_taken_once_and_eats_the_space_after_its_control_word() {
        // The compiler (pin `3d3d5ae3`) emits `Inline::HFill` inside titles,
        // through macro bodies too, and `Inline::TextGlue` for text-mode
        // `\quad`/`\qquad`; the pipeline must not add a second fill from
        // the macro body's bytes, and the whitespace after the control word
        // is TeX's to eat.
        let src = "\\documentclass[11pt]{article}\n\\newcommand{\\problem}[2]{\\subsection*{Problem #1 \\hfill \\normalfont[#2 points]}}\n\\begin{document}\n\\problem{1}{4}\n\\subsection*{Bonus \\hfill \\normalfont[1 pt]}\nA \\quad B\\qquad C.\n\\end{document}\n";
        let doc = adapt(&[src], 0, &flashtex_compiler::parser::parse(src), &RenderOptions::default(), &Labels::default());
        let shapes: Vec<String> = doc
            .blocks
            .iter()
            .map(|b| match b {
                Block::Heading { items, .. } => shape(items),
                Block::Paragraph { parts, .. } => parts
                    .iter()
                    .map(|p| match p {
                        ParaPart::Lines(items) => shape(items),
                        ParaPart::Display { .. } => "D".to_string(),
                    })
                    .collect(),
                Block::Rule { .. } => "R".to_string(),
            })
            .collect();
        // `Problem 1 \hfill \normalfont[4 points]`: one fill, no space after it.
        // `A \quad B\qquad C.`: the space before `\quad` stays, the one after
        // is eaten; `B\qquad` has none before.
        assert_eq!(shapes, ["WSWSFWSW", "WSFWSW", "WSQWQW"]);
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
