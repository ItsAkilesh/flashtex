//! Parser for the documented LaTeX subset.
//!
//! Honest boundary: this is a finite grammar, not TeX. It recognises the common
//! LaTeX preamble and implements bounded `\newcommand`/`\renewcommand`
//! expansion, but there is no category-code mutation, register, conditional,
//! package loading, or general environment implementation.

use std::collections::{BTreeMap, HashMap};

use crate::diagnostics::Diagnostic;
use crate::lexer::{apply_text_ligatures, tokenize, tokenize_document, Token, TokenKind};
use crate::math::{self, MathList};
use crate::theorems::{self, TheoremDef, TheoremStyle};
use crate::{DocumentId, Span};

mod tabular;

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
        style: TextStyle,
        /// Whether the source had real whitespace (or nothing — start of a
        /// paragraph/group) immediately before this run, as opposed to
        /// sitting directly against whatever came before it (the far side
        /// of `$...$`, or a macro-argument splice such as `\normalfont[#2
        /// points]` gluing the literal `[` to the substituted digits).
        /// Real TeX never inserts an inter-word gap that is not present in
        /// the source; see `layout::LayoutCursor::place`.
        space_before: bool,
    },
    LineBreak {
        span: Span,
    },
    /// Explicit text-mode horizontal glue (`\quad` is 1em, `\qquad` is 2em),
    /// measured in ems of the surrounding body text size. Named distinctly
    /// from `HSpace` below (a fixed-point `\hspace{<dimen>}` glue) since the
    /// two behave differently at a line break: this discardable glue mirrors
    /// TeX by breaking the line rather than overflowing it (see
    /// `layout::LayoutCursor::text_glue`).
    TextGlue {
        em: f64,
        span: Span,
    },
    Math {
        list: MathList,
        display: bool,
        number: Option<String>,
        number_span: Option<Span>,
        span: Span,
        /// See `Inline::Text::space_before`.
        space_before: bool,
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
    /// `\hfill`/`\hfil`: infinite horizontal stretch. Multiple fills on one
    /// line share the line's leftover width equally, as real TeX glue does;
    /// unlike TeX, `\hfil` and `\hfill` are not distinguished by stretch
    /// order (this layout has only one order of infinite glue), an accepted
    /// simplification. See `layout::LayoutCursor::resolve_hfill`.
    HFill {
        span: Span,
    },
    /// `\hspace{<dimen>}`/`\hspace*{<dimen>}`: a fixed, non-stretching space.
    /// `pt` is already converted (see `parse_dimen_pt`). Real TeX also lets
    /// plain `\hspace` glue (unlike the starred form) be discarded when it
    /// falls at a line break; this layout never discards glue at a line
    /// start, so both forms behave identically here.
    HSpace {
        pt: f64,
        span: Span,
    },
    /// `\footnote[<n>]{..}`, `\footnotemark[<n>]` or `\footnotetext[<n>]{..}`.
    /// `number` is the resolved `\thefootnote` (arabic). `span` is the
    /// command token, attributed to both superscript marks. `mark` is false
    /// only for `\footnotetext`; `text` is `None` only for `\footnotemark`.
    /// Page-bottom placement lives in `layout::footnotes`.
    Footnote {
        number: String,
        span: Span,
        mark: bool,
        text: Option<Vec<Inline>>,
        /// See `Inline::Text::space_before`.
        space_before: bool,
    },
    /// `tabular`/`tabular*`: an inline box (see `crate::tabular`).
    Tabular(Box<crate::tabular::Tabular>),
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
    /// One paragraph of an `itemize`/`enumerate` `\item`. `level` (1 =
    /// outermost) drives the hanging-indent margin; `label` carries the
    /// marker text and the `\item` span, and is `None` for a continuation
    /// paragraph of the same item (a blank line inside `\item`'s text) so the
    /// marker is not repeated while the hanging indent still applies.
    /// `extra_gap_before_pt`/`extra_gap_after_pt` are `\setlist`
    /// itemsep/topsep overrides (`0.0` without `\setlist`).
    ListItem {
        level: u8,
        label: Option<(String, Span)>,
        content: Vec<Inline>,
        /// Extra gap before this item, beyond the ordinary paragraph gap:
        /// `topsep` before the list's first item, `itemsep` before the rest.
        extra_gap_before_pt: f64,
        /// Extra gap after this item: `topsep`, set only on the list's last
        /// item.
        extra_gap_after_pt: f64,
        /// `\setlist{leftmargin=...}`'s effect on this level's own share of
        /// the cumulative hanging-indent margin (`Default` outside
        /// `\setlist`, or when the level's default `LIST_LEFTMARGIN_EM`
        /// share applies unchanged).
        leftmargin: ListLeftMargin,
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

/// `\setlist{leftmargin=...}`'s effect on a `Block::ListItem`'s own
/// contribution to the cumulative hanging-indent margin (see
/// `layout::list_margin_pt`); enclosing levels' shares are unaffected.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ListLeftMargin {
    /// No override: the level's default `LIST_LEFTMARGIN_EM` share applies.
    #[default]
    Default,
    /// `leftmargin=<dimen>`, already resolved to points.
    Explicit(f64),
    /// `leftmargin=*`: every distinct label text that can appear in this
    /// list, resolved once every `\item` in it has been seen (enumitem picks
    /// the widest of these once the labels are known — see `set_list`);
    /// `layout` measures each at the body size and adds `\labelsep`.
    Widest(Vec<String>),
}

/// Font selection for one text item, as set by `\textbf`, `\itshape`, etc.
/// Slanted shapes (`\textsl`, `\slshape`) are recorded as italic: the Core 14
/// faces have no slanted Times.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct TextStyle {
    pub bold: bool,
    pub italic: bool,
    pub family: TextFamily,
    /// The active `\tiny`..`\Huge` declaration, if any (`None` is
    /// `\normalsize`, the body size). Resolved to an actual point size in
    /// `layout::size_declaration_pt`, against the layout's own body size
    /// rather than here, since that is the one authoritative value.
    pub size: Option<FontSizeLevel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum TextFamily {
    #[default]
    Roman,
    Sans,
    Mono,
}

/// One `\tiny`..`\Huge` declaration level. Scoped on `TextStyle` exactly like
/// bold/italic/family, via the same group/environment style stack, rather
/// than a parallel size stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontSizeLevel {
    Tiny,
    ScriptSize,
    FootnoteSize,
    Small,
    /// `\large`.
    Large1,
    /// `\Large`.
    Large2,
    /// `\LARGE`.
    Large3,
    /// `\huge`.
    Huge1,
    /// `\Huge`.
    Huge2,
}

impl TextStyle {
    pub const BOLD: TextStyle = TextStyle {
        bold: true,
        italic: false,
        family: TextFamily::Roman,
        size: None,
    };
}

/// Argument-taking style commands (`\textbf{...}`).
fn style_command(name: &str) -> bool {
    matches!(
        name,
        "textbf"
            | "textmd"
            | "textit"
            | "textsl"
            | "textup"
            | "emph"
            | "texttt"
            | "textrm"
            | "textsf"
            | "textnormal"
    )
}

/// Group-scoped style declarations (`\bfseries`, `{\bf ...}`). `\tiny`..
/// `\Huge` are declarations too, not argument-taking commands: `\Large{...}`
/// (a common `\textbf{...}`-style misuse) is deliberately handled the same
/// way as `{\Large ...}` — its size stays active past the immediate group,
/// matching real LaTeX (the group only undoes assignments made *inside* it).
fn style_declaration(name: &str) -> bool {
    matches!(
        name,
        "bfseries"
            | "mdseries"
            | "itshape"
            | "slshape"
            | "upshape"
            | "ttfamily"
            | "rmfamily"
            | "sffamily"
            | "normalfont"
            | "em"
            | "bf"
            | "it"
            | "sl"
            | "tt"
            | "rm"
            | "sf"
            | "tiny"
            | "scriptsize"
            | "footnotesize"
            | "small"
            | "normalsize"
            | "large"
            | "Large"
            | "LARGE"
            | "huge"
            | "Huge"
    )
}

/// The style after applying one style command or declaration to `style`.
fn apply_style(style: TextStyle, name: &str) -> TextStyle {
    let mut next = style;
    match name {
        "textbf" | "bfseries" => next.bold = true,
        "textmd" | "mdseries" => next.bold = false,
        "textit" | "textsl" | "itshape" | "slshape" => next.italic = true,
        "textup" | "upshape" => next.italic = false,
        "emph" | "em" => next.italic = !style.italic,
        "texttt" | "ttfamily" => next.family = TextFamily::Mono,
        "textrm" | "rmfamily" => next.family = TextFamily::Roman,
        "textsf" | "sffamily" => next.family = TextFamily::Sans,
        "textnormal" | "normalfont" => next = TextStyle::default(),
        // LaTeX 2.09 forms reset the other attributes: `\bf` is
        // `\normalfont\bfseries`.
        "bf" => next = TextStyle::BOLD,
        "it" | "sl" => {
            next = TextStyle {
                italic: true,
                ..TextStyle::default()
            }
        }
        "tt" | "rm" | "sf" => next = apply_style(TextStyle::default(), &format!("{name}family")),
        "tiny" => next.size = Some(FontSizeLevel::Tiny),
        "scriptsize" => next.size = Some(FontSizeLevel::ScriptSize),
        "footnotesize" => next.size = Some(FontSizeLevel::FootnoteSize),
        "small" => next.size = Some(FontSizeLevel::Small),
        "normalsize" => next.size = None,
        "large" => next.size = Some(FontSizeLevel::Large1),
        "Large" => next.size = Some(FontSizeLevel::Large2),
        "LARGE" => next.size = Some(FontSizeLevel::Large3),
        "huge" => next.size = Some(FontSizeLevel::Huge1),
        "Huge" => next.size = Some(FontSizeLevel::Huge2),
        _ => {}
    }
    next
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
    /// Body size from a `10pt`/`11pt`/`12pt` `\documentclass` option.
    pub class_size_pt: Option<f64>,
    /// `\setlength{\parskip}{..}` from the preamble, in points.
    pub parskip_pt: Option<f64>,
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

impl Parsed {
    /// Layout constraints with the preamble's body size and `\parskip` applied.
    pub fn preamble_constraints(
        &self,
        constraints: crate::layout::LayoutConstraints,
    ) -> crate::layout::LayoutConstraints {
        crate::layout::LayoutConstraints {
            font_size_pt: self.class_size_pt.unwrap_or(constraints.font_size_pt),
            parskip_pt: self.parskip_pt.or(constraints.parskip_pt),
            ..constraints
        }
    }
}

const BUILT_INS: &[&str] = &[
    "section",
    "subsection",
    "textbf",
    "textmd",
    "emph",
    "textit",
    "textsl",
    "textup",
    "texttt",
    "textrm",
    "textsf",
    "textnormal",
    "begin",
    "end",
    "par",
    "documentclass",
    "setlength",
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
    "hfil",
    "hspace",
    "footnote",
    "footnotemark",
    "footnotetext",
    "normalfont",
    "bfseries",
    "mdseries",
    "itshape",
    "slshape",
    "upshape",
    "ttfamily",
    "rmfamily",
    "sffamily",
    "em",
    "bf",
    "it",
    "sl",
    "tt",
    "rm",
    "sf",
    "quad",
    "qquad",
    "bigskip",
    "medskip",
    "smallskip",
    "vspace",
    "hrule",
    "newpage",
    "pagestyle",
    "listfiles",
    "centering",
    "Centering",
    "raggedright",
    "RaggedRight",
    "raggedleft",
    "RaggedLeft",
    "noindent",
    "tiny",
    "scriptsize",
    "footnotesize",
    "small",
    "normalsize",
    "large",
    "Large",
    "LARGE",
    "huge",
    "Huge",
];

/// Parses a LaTeX dimension (`12pt`, `1.5em`, `0.5in`, `2cm`, `10mm`, `2ex`,
/// `12bp`) to points. `em`/`ex` are relative to the compiler's fixed body size
/// because the layout does not yet carry a current font size into dimension
/// parsing; `ex` uses the common TeX-metrics approximation of half an em,
/// since no real x-height is read from the font. `bp` ("big point") is exactly this compiler's own
/// internal point (both are 1/72 inch, matching the 612×792pt page in
/// `layout.rs`), unlike `in`/`cm`/`mm` below, which follow TeX's own
/// 72.27-per-inch point.
pub(crate) fn parse_dimen_pt(text: &str) -> Option<f64> {
    parse_dimen_pt_at(text, crate::layout::BODY_SIZE_PT)
}

/// `parse_dimen_pt` with `em`/`ex` relative to `body_pt`.
pub(crate) fn parse_dimen_pt_at(text: &str, body_pt: f64) -> Option<f64> {
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
        "pt" | "bp" => 1.0,
        "in" => 72.27,
        "cm" => 72.27 / 2.54,
        "mm" => 72.27 / 25.4,
        "em" => body_pt,
        "ex" => body_pt * 0.5,
        _ => return None,
    };
    Some(value * per_pt)
}

/// True when `content` (already trimmed) is safe for the unsupported-command
/// recovery policy to assume is a parameter rather than prose — see the
/// policy comment on `unsupported` below for the full rationale. A dimension
/// (reusing `parse_dimen_pt`, so `\vspace{0.6em}`-style values match) or a
/// single lowercase keyword (`empty`, `arabic`, ...) both qualify. A
/// single-*character* word is deliberately excluded: real LaTeX keyword
/// parameters are essentially always two or more letters (`empty`, `plain`,
/// `arabic`, ...), while a single lowercase letter is far more likely to be
/// genuine one-letter prose or a macro body (`\def\x{y}`) that must not be
/// silently dropped.
fn looks_like_recoverable_argument(content: &str) -> bool {
    parse_dimen_pt(content).is_some()
        || (content.chars().count() > 1 && content.chars().all(|ch| ch.is_ascii_lowercase()))
}

/// Plain TeX's conventional `\smallskipamount`/`\medskipamount`/
/// `\bigskipamount`, in points. Real TeX also gives each a `plus`/`minus`
/// stretch component; this layout model has no rubber lengths (see
/// `Block::VSpace`, which `\vspace` already feeds a flat point value), so
/// these are the flat amounts with the stretch/shrink honestly dropped.
const SMALL_SKIP_PT: f64 = 3.0;
const MEDIUM_SKIP_PT: f64 = 6.0;
const BIG_SKIP_PT: f64 = 12.0;

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

/// Whether the token at `index` in `tokens` sits directly against real
/// source whitespace — a preceding `TokenKind::Space`/`ParBreak` — or is the
/// first token, in which case there is nothing before it to glue against.
/// Any other neighbour (a word, a control word, `$`, a brace, ...) means the
/// source had no space there, so layout must not invent one. Shared by the
/// main token cursor (`P::space_precedes`) and `P::inlines_from_tokens`,
/// which walks its own flattened, macro-expanded token list.
fn preceded_by_space(tokens: &[InputToken], index: usize) -> bool {
    index == 0
        || matches!(
            tokens.get(index - 1).map(|input| &input.token.kind),
            Some(TokenKind::Space) | Some(TokenKind::ParBreak)
        )
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
        class_size_pt: None,
        parskip_pt: None,
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
        footnote_counter: 0,
        current_counter: None,
        seen_labels: HashMap::new(),
        list_stack: Vec::new(),
        pending_item_label: None,
        paragraph_styles: Vec::new(),
        document_global_state: false,
        style: TextStyle::default(),
        style_stack: Vec::new(),
        env_styles: Vec::new(),
        declared_alignment: None,
        alignment_stack: Vec::new(),
        env_alignments: Vec::new(),
        list_spacing: HashMap::new(),
        theorems: HashMap::new(),
        theorem_style: TheoremStyle::default(),
        theorem_counters: HashMap::new(),
    };
    // The kernel's `\def\arraystretch{1}`, so `\renewcommand` can change it.
    p.macros.insert(
        "arraystretch".into(),
        MacroDef {
            argument_count: 0,
            body: vec![Token {
                kind: TokenKind::Word("1".into()),
                span: Span::in_document(DocumentId(entry), 0, 0),
            }],
        },
    );
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
        class_size_pt: p.class_size_pt,
        parskip_pt: p.parskip_pt,
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
    class_size_pt: Option<f64>,
    parskip_pt: Option<f64>,
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
    /// LaTeX's `footnote` counter; article never resets it.
    footnote_counter: u32,
    current_counter: Option<String>,
    seen_labels: HashMap<String, Span>,
    /// Environment name, item count, an enumitem label template if given,
    /// the `\setlist` spacing resolved when this list's `\begin` ran, and the
    /// `blocks` length at that point (where this list's own items start, for
    /// the `leftmargin=*` backpatch once every item is known — see
    /// `environment`).
    list_stack: Vec<(String, u32, Option<String>, ListSpacing, usize)>,
    /// The marker text and span set by the most recent `\item`, consumed by
    /// the next `flush_paragraph`/`flush_list_item` call (its own paragraph,
    /// or a later one if the item's text is empty). `None` once consumed, so
    /// later paragraphs of the same item render with the hanging indent but
    /// no repeated label.
    pending_item_label: Option<(String, Span)>,
    paragraph_styles: Vec<ParagraphStyle>,
    document_global_state: bool,
    /// Current text style; saved on `{` and environment entry, restored on
    /// the matching `}` or `\end`.
    style: TextStyle,
    style_stack: Vec<TextStyle>,
    env_styles: Vec<TextStyle>,
    /// `\centering`/`\raggedright`/`\raggedleft` in force. Like TeX's
    /// paragraph parameters it is read when a paragraph ends, and it is
    /// saved on `{`/`\begin` and restored on the matching `}`/`\end`.
    declared_alignment: Option<ParagraphStyle>,
    alignment_stack: Vec<Option<ParagraphStyle>>,
    env_alignments: Vec<Option<ParagraphStyle>>,
    /// `\setlist` overrides, keyed by environment name ("itemize" /
    /// "enumerate"). A list resolves its spacing from here when `\begin`
    /// runs, so a later `\setlist` does not retroactively change an
    /// already-open list.
    list_spacing: HashMap<String, ListSpacing>,
    /// `\newtheorem` registrations, keyed by environment name.
    theorems: HashMap<String, TheoremDef>,
    /// The style set by the most recent `\theoremstyle`, applied to
    /// `\newtheorem` declarations from that point on (`plain` until then,
    /// matching amsthm's own default).
    theorem_style: TheoremStyle,
    /// Theorem counters, keyed by `TheoremDef::counter` (an environment's
    /// own name, or the name of the environment whose counter it shares).
    theorem_counters: HashMap<String, u32>,
}

/// Extra vertical space `\setlist{itemsep=...,topsep=...}` adds on top of
/// the compiler's ordinary paragraph gap. Both default to zero, matching
/// today's spacing exactly when `\setlist` is never called.
#[derive(Debug, Clone, Copy, Default)]
struct ListSpacing {
    itemsep_pt: f64,
    topsep_pt: f64,
    leftmargin: LeftMarginSetting,
}

/// `\setlist{leftmargin=...}`'s value, resolved into a `Block::ListItem`'s
/// `ListLeftMargin` once the list's items are known (`Widest` needs every
/// label; `Explicit` is applied to each item as it is created).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum LeftMarginSetting {
    #[default]
    Unset,
    /// `leftmargin=<dimen>`, already resolved to points.
    Explicit(f64),
    /// `leftmargin=*`.
    Widest,
}

impl P<'_> {
    fn peek(&self) -> Option<&Token> {
        self.t.get(self.i).map(|t| &t.token)
    }

    /// See the free function `preceded_by_space`, applied to the main token
    /// cursor.
    fn space_precedes(&self, index: usize) -> bool {
        preceded_by_space(&self.t, index)
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
                    let space_before = self.space_precedes(self.i);
                    self.i += 1;
                    if render {
                        para.push(Inline::Text {
                            text: apply_text_ligatures(&word),
                            span: tok.span,
                            style: self.style,
                            space_before,
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
                    self.open_group(tok.span);
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
                        if let Some(style) = self.style_stack.pop() {
                            self.style = style;
                        }
                        if let Some(alignment) = self.alignment_stack.pop() {
                            self.declared_alignment = alignment;
                        }
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
            "setlength" => self.set_length(span),
            "usepackage" => self.use_package(span),
            "setlist" => self.set_list(span),
            "newcommand" | "renewcommand" => self.define_macro(name, span),
            "newtheorem" => self.new_theorem(span),
            "theoremstyle" => self.set_theorem_style(span),
            "begin" | "end" => self.environment(name, span, blocks, para),
            "input" | "include" => self.include(name, span, blocks, para),
            // MacTeX writes package-version banners to the log for `\listfiles`;
            // this compiler has no log stream to write them to, so the honest
            // behaviour is a documented no-op rather than an "unsupported"
            // diagnostic for a command every corpus fixture's preamble carries.
            "listfiles" => {}
            // Alignment declarations (ragged2e's capitalised forms differ only
            // in hyphenation, which this compiler does not do). Handled before
            // the preamble guard because a global `\raggedright` there is
            // ordinary LaTeX.
            "centering" | "Centering" => self.declared_alignment = Some(ParagraphStyle::Center),
            "raggedright" | "RaggedRight" => {
                self.declared_alignment = Some(ParagraphStyle::FlushLeft)
            }
            "raggedleft" | "RaggedLeft" => {
                self.declared_alignment = Some(ParagraphStyle::FlushRight)
            }
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
                    theorems::reset_within_section(&self.theorems, &mut self.theorem_counters);
                    self.section_counter.to_string()
                } else {
                    self.subsection_counter += 1;
                    format!("{}.{}", self.section_counter, self.subsection_counter)
                };
                if !starred {
                    self.current_counter = Some(number.clone());
                }
                let content = self.inlines_from_tokens(tokens, TextStyle::BOLD);
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
                    let style = self.style;
                    para.extend(self.inlines_from_tokens(tokens, style));
                } else {
                    self.flush_paragraph(blocks, para);
                    self.figure_counter += 1;
                    self.current_counter = Some(self.figure_counter.to_string());
                    let mut content = vec![Inline::Text {
                        text: format!("Figure {}:", self.figure_counter),
                        span,
                        style: TextStyle::default(),
                        space_before: true,
                    }];
                    content.extend(self.inlines_from_tokens(tokens, TextStyle::default()));
                    blocks.push(Block::FigureCaption { content });
                    self.finish_block_dependencies();
                }
            }
            "item" => {
                let gap_before = self
                    .list_stack
                    .last()
                    .map(|(_, count, _, spacing, _)| {
                        if *count <= 1 {
                            spacing.topsep_pt
                        } else {
                            spacing.itemsep_pt
                        }
                    })
                    .unwrap_or(0.0);
                self.flush_list_item(blocks, para, gap_before, 0.0);
                match self.list_stack.last_mut() {
                    Some((kind, count, template, _, _)) => {
                        *count += 1;
                        let marker = if kind == "enumerate" {
                            match template {
                                Some(template) => enumitem_label(template, *count),
                                None => format!("{}.", count),
                            }
                        } else {
                            "•".to_string()
                        };
                        self.pending_item_label = Some((marker, span));
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
            _ if style_command(name) => {
                self.skip_spaces();
                let next = apply_style(self.style, name);
                if let Some(open) = self.closed_group_start() {
                    // Re-enter the argument as an ordinary group so math and
                    // other commands inside it are parsed normally.
                    self.i += 1;
                    self.open_group(open);
                    self.style = next;
                } else {
                    let (tokens, _) = self.required_group(name, span);
                    para.extend(self.inlines_from_tokens(tokens, next));
                }
            }
            _ if style_declaration(name) => self.style = apply_style(self.style, name),
            "hfill" | "hfil" => para.push(Inline::HFill { span }),
            "footnote" | "footnotemark" | "footnotetext" => self.footnote(name, span, para),
            "hspace" => {
                // The star only affects whether the glue survives being
                // discarded at a line break in real TeX, which this layout
                // never does anyway (see the `Inline::HSpace` comment), so
                // both forms are parsed identically.
                let _starred = self.take_optional_star();
                let (tokens, argument_span) = self.required_group(name, span);
                let raw = token_text(&tokens);
                match parse_dimen_pt(&raw) {
                    Some(pt) => para.push(Inline::HSpace {
                        pt,
                        span: span.merge(argument_span),
                    }),
                    None => self.diags.push(Diagnostic::error(
                        format!(
                            "\\hspace requires a recognised dimension, got '{}'",
                            raw.trim()
                        ),
                        Some(span.merge(argument_span)),
                        Some("ignored the malformed \\hspace argument".into()),
                    )),
                }
            }
            // No paragraph is ever given a first-line indent in this layout
            // model, so there is nothing for \noindent to suppress: an honest
            // no-op rather than a fabricated indent to cancel.
            "noindent" => {}
            // Text-mode horizontal glue. `\quad`/`\qquad` are also implemented
            // in math mode (`src/math.rs`); this arm covers the same commands
            // used directly in running text, 1em/2em of the body text size.
            "quad" => para.push(Inline::TextGlue {
                em: math::QUAD_EM,
                span,
            }),
            "qquad" => para.push(Inline::TextGlue {
                em: 2.0 * math::QUAD_EM,
                span,
            }),
            "par" => self.flush_paragraph(blocks, para),
            "bigskip" | "medskip" | "smallskip" => {
                let pt = match name {
                    "bigskip" => BIG_SKIP_PT,
                    "medskip" => MEDIUM_SKIP_PT,
                    _ => SMALL_SKIP_PT,
                };
                self.flush_paragraph(blocks, para);
                blocks.push(Block::VSpace { pt });
                self.finish_block_dependencies();
            }
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
        let options = self.optional_bracket_argument();
        if self.class_size_pt.is_none() {
            self.class_size_pt = options.and_then(|(options, _)| {
                options.split(',').find_map(|option| match option.trim() {
                    "10pt" => Some(10.0),
                    "11pt" => Some(11.0),
                    "12pt" => Some(12.0),
                    _ => None,
                })
            });
        }
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

    /// `\setlength{\parskip}{..}` and `\setlength{\parindent}{..}` in the
    /// preamble. `em`/`ex` resolve against the class body size. This engine
    /// never indents paragraphs, so only a zero `\parindent` is exact.
    fn set_length(&mut self, span: Span) {
        let (target_tokens, _) = self.required_group("setlength", span);
        let (value_tokens, value_span) = self.required_group("setlength", span);
        let span = span.merge(value_span);
        let target = token_text(&target_tokens)
            .trim()
            .trim_start_matches('\\')
            .to_string();
        let raw = token_text(&value_tokens);
        let body = self.class_size_pt.unwrap_or(crate::layout::BODY_SIZE_PT);
        let Some(pt) = parse_dimen_pt_at(&raw, body) else {
            self.diags.push(Diagnostic::error(
                format!(
                    "\\setlength requires a recognised dimension, got '{}'",
                    raw.trim()
                ),
                Some(span),
                Some("ignored the length assignment".into()),
            ));
            return;
        };
        let in_preamble = self.has_document && !self.in_body;
        match target.as_str() {
            "parskip" if in_preamble => self.parskip_pt = Some(pt),
            "parindent" if in_preamble && pt == 0.0 => {}
            "parindent" if in_preamble => self.diags.push(Diagnostic::warning(
                "\\parindent is recognised but paragraph indentation is not implemented",
                Some(span),
                Some("paragraphs are not indented".into()),
            )),
            _ => self.diags.push(Diagnostic::warning(
                format!(
                    "\\setlength{{\\{}}} is recognised but not implemented here",
                    target
                ),
                Some(span),
                Some("ignored the length assignment".into()),
            )),
        }
    }

    /// `\setlist[<env list>]{key=value,...}`: enumitem's list-spacing
    /// override. The optional argument names which environments the given
    /// keys apply to (a comma list; omitted means every list). `itemsep`,
    /// `topsep` and `leftmargin` (an explicit dimension, or `*`) change
    /// layout; every other recognised enumitem key (`label`, `parsep`,
    /// `partopsep`, ...) has no equivalent in this layout engine and is
    /// reported once, by name.
    fn set_list(&mut self, span: Span) {
        let environments = self
            .optional_bracket_argument()
            .map(|(options, _)| options)
            .unwrap_or_default();
        let (tokens, argument_span) = self.required_group("setlist", span);
        let full_span = span.merge(argument_span);
        let envs: Vec<String> = if environments.trim().is_empty() {
            vec!["itemize".to_string(), "enumerate".to_string()]
        } else {
            environments
                .split(',')
                .map(str::trim)
                .filter(|env| !env.is_empty())
                .map(str::to_string)
                .collect()
        };

        let mut itemsep_pt = None;
        let mut topsep_pt = None;
        let mut leftmargin = None;
        let mut ignored_keys: Vec<String> = Vec::new();
        for pair in token_text(&tokens).split(',') {
            let pair = pair.trim();
            if pair.is_empty() {
                continue;
            }
            let (key, value) = match pair.split_once('=') {
                Some((key, value)) => (key.trim(), Some(value.trim())),
                None => (pair, None),
            };
            match key {
                "itemsep" if value.and_then(parse_dimen_pt).is_some() => {
                    itemsep_pt = value.and_then(parse_dimen_pt);
                }
                "topsep" if value.and_then(parse_dimen_pt).is_some() => {
                    topsep_pt = value.and_then(parse_dimen_pt);
                }
                "leftmargin" if value == Some("*") => {
                    leftmargin = Some(LeftMarginSetting::Widest);
                }
                "leftmargin" if value.and_then(parse_dimen_pt).is_some() => {
                    leftmargin = value
                        .and_then(parse_dimen_pt)
                        .map(LeftMarginSetting::Explicit);
                }
                _ if !ignored_keys.iter().any(|seen| seen == key) => {
                    ignored_keys.push(key.to_string());
                }
                _ => {}
            }
        }

        for env in &envs {
            let spacing = self.list_spacing.entry(env.clone()).or_default();
            if let Some(pt) = itemsep_pt {
                spacing.itemsep_pt = pt;
            }
            if let Some(pt) = topsep_pt {
                spacing.topsep_pt = pt;
            }
            if let Some(lm) = leftmargin {
                spacing.leftmargin = lm;
            }
        }

        if !ignored_keys.is_empty() {
            ignored_keys.sort();
            self.diags.push(Diagnostic::warning(
                format!(
                    "\\setlist keys {} are recognised but not implemented",
                    ignored_keys.join(", ")
                ),
                Some(full_span),
                Some("lists use the compiler's default spacing for these keys".into()),
            ));
        }
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
                "gather"
                    | "gather*"
                    | "align"
                    | "align*"
                    | "alignat"
                    | "alignat*"
                    | "flalign"
                    | "flalign*"
                    | "multline"
                    | "multline*"
            ) && self.in_body
            {
                self.multirow_environment(span, &environment, blocks, para);
                return;
            }
            if matches!(environment.as_str(), "tabular" | "tabular*") && self.in_body {
                self.tabular_environment(span, &environment, para);
                return;
            }
            self.env_alignments.push(self.declared_alignment);
            if environment == "document" && self.has_document {
                self.in_body = true;
            } else if environment == "figure" && self.in_body {
                self.flush_paragraph(blocks, para);
            } else if let (Some(style), true) = (paragraph_style(&environment), self.in_body) {
                self.flush_paragraph(blocks, para);
                self.paragraph_styles.push(style);
                // An inner alignment environment overrides an outer declaration.
                if style != ParagraphStyle::Quote {
                    self.declared_alignment = None;
                }
            } else if matches!(environment.as_str(), "itemize" | "enumerate") && self.in_body {
                self.flush_paragraph(blocks, para);
                let template = self.optional_bracket_argument().map(|(options, _)| options);
                let spacing = self
                    .list_spacing
                    .get(&environment)
                    .copied()
                    .unwrap_or_default();
                self.list_stack
                    .push((environment.clone(), 0, template, spacing, blocks.len()));
            } else if self.in_body
                && (self.theorems.contains_key(&environment) || environment == "proof")
            {
                self.flush_paragraph(blocks, para);
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
                .push((environment.clone(), span.merge(argument_span)));
            self.env_styles.push(self.style);
            if self.in_body {
                if let Some(theorem) = self.theorems.get(&environment).cloned() {
                    self.begin_theorem(&theorem, span, para);
                } else if environment == "proof" {
                    self.begin_proof(span, para);
                }
            }
            return;
        }

        let popped = self.env_stack.pop();
        let had_open_environment = popped.is_some();
        if popped.is_some() {
            if let Some(style) = self.env_styles.pop() {
                self.style = style;
            }
        }
        match popped {
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
            let (gap_before, gap_after) = match self.list_stack.last() {
                Some((_, count, _, spacing, _)) => (
                    if *count <= 1 {
                        spacing.topsep_pt
                    } else {
                        spacing.itemsep_pt
                    },
                    spacing.topsep_pt,
                ),
                None => (0.0, 0.0),
            };
            self.flush_list_item(blocks, para, gap_before, gap_after);
            let level = self.list_stack.len() as u8;
            if let Some((kind, count, template, spacing, start)) = self.list_stack.pop() {
                if spacing.leftmargin == LeftMarginSetting::Widest && count > 0 {
                    let labels: Vec<String> = if kind == "enumerate" {
                        // An alphabetic counter has only 26 possible single-
                        // letter values, so enumitem checks every one of them
                        // regardless of how many items this particular list
                        // has; other styles use this list's own item count
                        // (its labels only grow wider as the count does).
                        let widest_count = match &template {
                            Some(t) if matches!(enumitem_label_style(t), 'a' | 'A') => 26,
                            _ => count,
                        };
                        (1..=widest_count)
                            .map(|n| match &template {
                                Some(template) => enumitem_label(template, n),
                                None => format!("{n}."),
                            })
                            .collect()
                    } else {
                        vec!["•".to_string()]
                    };
                    for block in &mut blocks[start..] {
                        if let Block::ListItem {
                            level: item_level,
                            leftmargin,
                            ..
                        } = block
                        {
                            if *item_level == level && matches!(leftmargin, ListLeftMargin::Default)
                            {
                                *leftmargin = ListLeftMargin::Widest(labels.clone());
                            }
                        }
                    }
                }
            }
        } else if environment == "figure" || self.theorems.contains_key(&environment) {
            self.flush_paragraph(blocks, para);
        } else if environment == "proof" {
            para.push(Inline::HFill { span });
            para.push(Inline::Text {
                text: "∎".to_string(),
                span,
                style: TextStyle::default(),
                space_before: false,
            });
            self.flush_paragraph(blocks, para);
        }
        if environment == "document" && self.has_document {
            self.flush_paragraph(blocks, para);
            self.in_body = false;
            self.document_ended = true;
        }
        // Restored only after the flushes above: environments that end their
        // paragraph do so while their own declarations are still in force.
        if had_open_environment {
            if let Some(alignment) = self.env_alignments.pop() {
                self.declared_alignment = alignment;
            }
        }
    }

    /// `\newtheorem{name}{Title}`, its starred (unnumbered) form, the
    /// shared-counter form `\newtheorem{name}[shared]{Title}`, and the
    /// reset-on-section form `\newtheorem{name}{Title}[section]`. See
    /// `theorems::TheoremDef`.
    fn new_theorem(&mut self, span: Span) {
        let starred = self.take_optional_star();
        let (name_tokens, name_span) = self.required_group("newtheorem", span);
        let name = token_text(&name_tokens).trim().to_string();
        let shared = self.optional_bracket_argument();
        let (title_tokens, _) = self.required_group("newtheorem", span);
        let title = token_text(&title_tokens).trim().to_string();
        let within = if shared.is_none() {
            self.optional_bracket_argument()
        } else {
            None
        };
        if name.is_empty() {
            self.diags.push(Diagnostic::error(
                "\\newtheorem was given an empty environment name",
                Some(span.merge(name_span)),
                Some("ignored the declaration".into()),
            ));
            return;
        }
        let (counter, within_section) = match shared {
            Some((shared_name, shared_span)) => {
                let shared_name = shared_name.trim().to_string();
                match self.theorems.get(&shared_name) {
                    Some(existing) => (existing.counter.clone(), existing.within_section),
                    None => {
                        self.diags.push(Diagnostic::error(
                            format!(
                                "\\newtheorem{{{name}}}[{shared_name}] shares the counter of \
                                 undefined theorem environment '{shared_name}'"
                            ),
                            Some(shared_span),
                            Some("ignored the declaration".into()),
                        ));
                        return;
                    }
                }
            }
            None => {
                let within_section = match within {
                    None => false,
                    Some((counter_name, _)) if counter_name.trim() == "section" => true,
                    Some((counter_name, counter_span)) => {
                        let counter_name = counter_name.trim().to_string();
                        self.diags.push(Diagnostic::warning(
                            format!(
                                "\\newtheorem counter '[{counter_name}]' is recognised but not implemented"
                            ),
                            Some(counter_span),
                            Some(format!(
                                "'{name}' is numbered without resetting on '{counter_name}'"
                            )),
                        ));
                        false
                    }
                };
                (name.clone(), within_section)
            }
        };
        self.theorems.insert(
            name,
            TheoremDef {
                title,
                style: self.theorem_style,
                numbered: !starred,
                counter,
                within_section,
            },
        );
    }

    fn set_theorem_style(&mut self, span: Span) {
        let (tokens, argument_span) = self.required_group("theoremstyle", span);
        let name = token_text(&tokens).trim().to_string();
        match TheoremStyle::from_name(&name) {
            Some(style) => self.theorem_style = style,
            None => self.diags.push(Diagnostic::error(
                format!("\\theoremstyle{{{name}}} is not a recognised amsthm style"),
                Some(span.merge(argument_span)),
                Some("kept the previous \\theoremstyle in effect".into()),
            )),
        }
    }

    /// The head run and, for numbered environments, the counter for
    /// entering a `\newtheorem`-registered environment. Called after
    /// `self.style` has already been saved onto `env_styles` by the caller
    /// (see `environment`), so mutating it here to the body's default style
    /// is correctly restored at the matching `\end`.
    fn begin_theorem(&mut self, def: &TheoremDef, span: Span, para: &mut Vec<Inline>) {
        let note = self.optional_bracket_argument();
        let mut head = def.title.clone();
        if def.numbered {
            let counter = self
                .theorem_counters
                .entry(def.counter.clone())
                .or_insert(0);
            *counter += 1;
            let n = *counter;
            let number = if def.within_section {
                format!("{}.{}", self.section_counter, n)
            } else {
                n.to_string()
            };
            self.current_counter = Some(number.clone());
            head.push(' ');
            head.push_str(&number);
        }
        para.push(Inline::Text {
            text: head,
            span,
            style: def.style.head_style(),
            space_before: true,
        });
        if let Some((note_text, note_span)) = note {
            let note_text = note_text.trim();
            if !note_text.is_empty() {
                para.push(Inline::Text {
                    text: format!(" ({note_text})"),
                    span: note_span,
                    style: TextStyle::default(),
                    space_before: false,
                });
            }
        }
        para.push(Inline::Text {
            text: ".".to_string(),
            span,
            style: TextStyle::default(),
            space_before: false,
        });
        self.style = def.style.body_style();
    }

    /// `proof`'s italic "Proof." head (or a custom `[...]` heading, still
    /// period-terminated) and upright body. The closing "∎" is appended by
    /// `environment`'s `\end` handling, once the body's last paragraph is
    /// known.
    fn begin_proof(&mut self, span: Span, para: &mut Vec<Inline>) {
        let heading = self
            .optional_bracket_argument()
            .map(|(text, _)| text.trim().to_string())
            .filter(|text| !text.is_empty())
            .unwrap_or_else(|| "Proof".to_string());
        para.push(Inline::Text {
            text: format!("{heading}."),
            span,
            style: TextStyle {
                italic: true,
                ..TextStyle::default()
            },
            space_before: true,
        });
        self.style = TextStyle::default();
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
            // Always its own line (see `layout::LayoutCursor::display_math`),
            // so whether real source whitespace preceded it is moot.
            space_before: true,
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
        let aligned = name.starts_with("align") || name.starts_with("flalign");
        if name.starts_with("alignat") {
            // The column-pair count; cells are split on `&` regardless.
            let _ = self.required_group("alignat", open);
        }
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
        if name == "multline" {
            // One multline display carries a single number, on its last line.
            let last = rows.len().saturating_sub(1);
            for (index, row) in rows.iter_mut().enumerate() {
                row.1 |= index != last;
            }
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
        let space_before = self.space_precedes(self.i);
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
            space_before,
            para,
        );
    }

    fn bracket_math(&mut self, open: Span, para: &mut Vec<Inline>) {
        let space_before = self.space_precedes(self.i);
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
            space_before,
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
        space_before: bool,
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
            space_before,
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

    /// The `{` span when the next token opens a group that closes in this
    /// token stream. Unclosed arguments keep `required_group`'s diagnostics.
    fn closed_group_start(&self) -> Option<Span> {
        let open = self
            .peek()
            .filter(|token| token.kind == TokenKind::LBrace)?;
        let mut depth = 0usize;
        for input in &self.t[self.i..] {
            match input.token.kind {
                TokenKind::LBrace => depth += 1,
                TokenKind::RBrace => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(open.span);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn open_group(&mut self, span: Span) {
        self.brace_stack.push(span);
        self.macro_scopes.push(HashMap::new());
        self.style_stack.push(self.style);
        self.alignment_stack.push(self.declared_alignment);
    }

    fn inlines_from_tokens(&mut self, tokens: Vec<InputToken>, base: TextStyle) -> Vec<Inline> {
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
        let mut style = base;
        let mut saved = Vec::new();
        let mut pending = None;
        for (index, input) in expanded.iter().enumerate() {
            let space_before = preceded_by_space(&expanded, index);
            match &input.token.kind {
                TokenKind::Command(name) if style_command(name) => {
                    pending = Some(apply_style(style, name));
                }
                TokenKind::Command(name) if style_declaration(name) => {
                    style = apply_style(style, name);
                }
                TokenKind::LBrace => {
                    saved.push(style);
                    if let Some(next) = pending.take() {
                        style = next;
                    }
                }
                TokenKind::RBrace => {
                    if let Some(previous) = saved.pop() {
                        style = previous;
                    }
                }
                TokenKind::Word(text) => content.push(Inline::Text {
                    text: apply_text_ligatures(text),
                    span: input.token.span,
                    style,
                    space_before,
                }),
                TokenKind::LineBreak => content.push(Inline::LineBreak {
                    span: input.token.span,
                }),
                // `\hfill`/`\hfil` take no argument, so — unlike `\hspace`,
                // which needs a following brace group this flat,
                // one-token-at-a-time pass has no way to consume — they fit
                // here directly. This is what makes `\problem`-style macro
                // bodies like `\subsection*{Problem #1 \hfill [#2 points]}`
                // (see the `problem_style_macro...` test below) right-flush:
                // heading/caption/`\textbf`-style content all reach the page
                // through this function rather than through `command`'s
                // ordinary dispatch. General nested-command dispatch inside
                // that content remains out of scope, per the module doc
                // comment.
                TokenKind::Command(name) if name == "hfill" || name == "hfil" => {
                    content.push(Inline::HFill {
                        span: input.token.span,
                    })
                }
                _ => {}
            }
        }
        content
    }

    /// `\footnote`, `\footnotemark` and `\footnotetext`, following latex.ltx:
    /// without `[<n>]`, `\footnote`/`\footnotemark` step the counter and
    /// `\footnotetext` reuses its current value; with `[<n>]` none of them
    /// step it. The footnote counter and page-bottom placement are
    /// document-global, so incremental block reuse is disabled (the same
    /// conservative rule `\label`/`\ref` use).
    fn footnote(&mut self, name: &str, span: Span, para: &mut Vec<Inline>) {
        let space_before = self.space_precedes(self.i - 1);
        self.document_global_state = true;
        let explicit = self
            .optional_bracket_argument()
            .and_then(|(raw, raw_span)| {
                let parsed = raw.trim().parse::<u32>().ok();
                if parsed.is_none() {
                    self.diags.push(Diagnostic::warning(
                        format!(
                            "\\{name} optional argument '{}' is not a number",
                            raw.trim()
                        ),
                        Some(raw_span),
                        Some("numbered the footnote from the footnote counter instead".into()),
                    ));
                }
                parsed
            });
        let number = match explicit {
            Some(number) => number,
            None if name == "footnotetext" => self.footnote_counter,
            None => {
                self.footnote_counter += 1;
                self.footnote_counter
            }
        };
        let text = if name == "footnotemark" {
            None
        } else {
            let (tokens, _) = self.required_group(name, span);
            Some(self.footnote_inlines(tokens, span))
        };
        para.push(Inline::Footnote {
            number: number.to_string(),
            span,
            mark: name != "footnotetext",
            text,
            space_before,
        });
    }

    /// Parses a footnote argument with the ordinary dispatch, so math, style
    /// commands and macros work inside it. The text starts from
    /// `\normalfont` (`\@footnotetext` resets the font). Paragraph breaks
    /// inside the argument become line breaks: the footnote is one inline
    /// sequence, not separate blocks; each break is attributed to `span`.
    fn footnote_inlines(&mut self, tokens: Vec<InputToken>, span: Span) -> Vec<Inline> {
        let outer_tokens = std::mem::replace(&mut self.t, tokens);
        let outer_index = std::mem::replace(&mut self.i, 0);
        let outer_style = std::mem::take(&mut self.style);
        let outer_label = self.pending_item_label.take();
        let outer_dependency_blocks = self.block_dependencies.len();
        // The argument is a TeX group: definitions inside it stay local.
        self.macro_scopes.push(HashMap::new());
        let mut blocks = Vec::new();
        let mut para = Vec::new();
        self.parse_stream(&mut blocks, &mut para);
        self.flush_paragraph(&mut blocks, &mut para);
        self.restore_scope();
        self.block_dependencies.truncate(outer_dependency_blocks);
        self.t = outer_tokens;
        self.i = outer_index;
        self.style = outer_style;
        self.pending_item_label = outer_label;

        let mut content: Vec<Inline> = Vec::new();
        for block in blocks {
            let inlines = match block {
                Block::Paragraph(inlines)
                | Block::Styled {
                    content: inlines, ..
                }
                | Block::ListItem {
                    content: inlines, ..
                } => inlines,
                _ => continue,
            };
            if !content.is_empty() && !inlines.is_empty() {
                content.push(Inline::LineBreak { span });
            }
            content.extend(inlines);
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
        self.flush_list_item(blocks, paragraph, 0.0, 0.0);
    }

    /// Flushes the accumulated paragraph. Inside a list, this attaches the
    /// pending `\item` marker (for the first paragraph of an item; later
    /// paragraphs of the same item get the hanging indent without repeating
    /// it), the item's nesting level, and any `\setlist` itemsep/topsep gap
    /// due before or after it (`0.0`/`0.0` from `flush_paragraph`, meaning no
    /// override — mid-item paragraph breaks never get itemsep/topsep, which
    /// are gaps between items, not between paragraphs within one). Falls
    /// back to an ordinary `Block::Paragraph`/`Block::Styled` outside a list.
    fn flush_list_item(
        &mut self,
        blocks: &mut Vec<Block>,
        paragraph: &mut Vec<Inline>,
        extra_gap_before_pt: f64,
        extra_gap_after_pt: f64,
    ) {
        let label = self.pending_item_label.take();
        if paragraph.is_empty() && label.is_none() {
            return;
        }
        let content = std::mem::take(paragraph);
        // A list level is "current" only once its first `\item` has been
        // seen (`count > 0`); text typed directly inside `itemize`/
        // `enumerate` before any `\item` falls back to an ordinary
        // paragraph, same as before this paragraph became list-aware.
        let list_level = self
            .list_stack
            .last()
            .filter(|(_, count, _, _, _)| *count > 0)
            .map(|_| self.list_stack.len() as u8);
        // `leftmargin=*` needs every item's label, so it is resolved later
        // (backpatched once the list's `\end` is reached — see
        // `environment`); an explicit dimension is already known.
        let leftmargin = match self.list_stack.last() {
            Some((_, _, _, spacing, _)) => match spacing.leftmargin {
                LeftMarginSetting::Explicit(pt) => ListLeftMargin::Explicit(pt),
                LeftMarginSetting::Unset | LeftMarginSetting::Widest => ListLeftMargin::Default,
            },
            None => ListLeftMargin::Default,
        };
        blocks.push(match list_level {
            Some(level) => Block::ListItem {
                level,
                label,
                content,
                extra_gap_before_pt,
                extra_gap_after_pt,
                leftmargin,
            },
            None => match (self.paragraph_styles.last(), self.declared_alignment) {
                // A declaration inside `quote` would otherwise drop its indent.
                (Some(&ParagraphStyle::Quote), _) => Block::Styled {
                    style: ParagraphStyle::Quote,
                    content,
                },
                (_, Some(style)) | (Some(&style), None) => Block::Styled { style, content },
                (None, None) => Block::Paragraph(content),
            },
        });
        self.finish_block_dependencies();
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

    /// Recovery policy for a command this compiler does not implement.
    ///
    /// The diagnostic naming the command must always survive — that is the
    /// contract that lets an author discover the gap; it is never hidden or
    /// weakened by what follows. What varies is only whether the following
    /// brace/bracket argument is also consumed. Left alone, the main token
    /// loop just keeps walking: a `{` opens an anonymous group and its
    /// contents fall through to ordinary paragraph text, so the argument
    /// itself becomes visible body text (e.g. `\vspace{0.6em}` used to leak
    /// the word "0.6em" onto the page, before `\vspace` gained its own
    /// implementation). That is fine — even correct — for a command whose
    /// argument IS meant to be read as prose: an unknown macro someone typoed,
    /// `\mycommand{Some real sentence}`, must keep that sentence visible, or
    /// the recovery would silently eat the author's content.
    ///
    /// So the argument is only skipped when it is conservatively safe to
    /// assume it is a parameter, not prose:
    ///   1. `name` is in `KNOWN_ARITY_UNIMPLEMENTED`: a command this compiler
    ///      recognises by name as taking a fixed count of non-prose
    ///      arguments it does not yet implement. Its whole arity is consumed
    ///      unconditionally — the command name alone is enough context.
    ///   2. Otherwise, for a genuinely unrecognised command, only the ONE
    ///      immediately following `{...}` group is inspected, and only
    ///      consumed if its full (trimmed) contents look like a dimension or
    ///      a keyword — see `looks_like_recoverable_argument`. Anything else
    ///      (multiple words, punctuation, a capitalized word, a lone letter)
    ///      is left in place and typeset as text, exactly as before.
    ///
    /// Either way, the diagnostic's recovery note records whether an argument
    /// was skipped, so the choice itself stays auditable from the output.
    fn unsupported(&mut self, name: &str, span: Span) {
        debug_assert!(!BUILT_INS.contains(&name));
        let skipped = self.skip_recoverable_argument(name);
        self.diags.push(Diagnostic::error(
            format!(
                "\\{} is not supported by this compiler version; unrestricted TeX math mode is not implemented",
                name
            ),
            Some(span),
            Some(if skipped {
                "skipped the command and its argument, which looked like a parameter rather than text".into()
            } else {
                "skipped the command; any braced argument was typeset as plain text".into()
            }),
        ));
    }

    /// Commands this compiler recognises by name as taking a fixed count of
    /// non-prose arguments it does not implement. Each listed argument is
    /// always skipped, regardless of content — the command name alone gives
    /// enough context to know the text was never meant to reach the page.
    /// Deliberately excludes `\vspace`/`\hrule`/`\newpage`/`\pagestyle` (and
    /// `\Large`/`\setlength`): those already have, or are gaining, their own
    /// real implementations elsewhere, so hardcoding them here would fight
    /// that work instead of falling out of it automatically.
    fn skip_recoverable_argument(&mut self, name: &str) -> bool {
        const KNOWN_ARITY_UNIMPLEMENTED: &[(&str, usize)] = &[
            // `\linespread{1.5}`: a bare scale factor with no unit suffix, so
            // the dimension heuristic below would never catch it on its own.
            ("linespread", 1),
        ];
        if let Some(&(_, arity)) = KNOWN_ARITY_UNIMPLEMENTED
            .iter()
            .find(|(known, _)| *known == name)
        {
            let mut skipped_any = false;
            for _ in 0..arity {
                if self.try_skip_braced_group(None) {
                    skipped_any = true;
                } else {
                    break;
                }
            }
            return skipped_any;
        }
        self.try_skip_braced_group(Some(looks_like_recoverable_argument))
    }

    /// Skips one `{...}` group immediately ahead (after whitespace), if one
    /// is there — and, when `predicate` is given, only when the group's
    /// trimmed text content satisfies it. Never emits a diagnostic of its
    /// own and never advances past anything on a rejected attempt: the
    /// missing- or non-matching-argument case is silent by design, since the
    /// ordinary token loop is what typesets it as text afterwards.
    fn try_skip_braced_group(&mut self, predicate: Option<fn(&str) -> bool>) -> bool {
        let mut cursor = self.i;
        while matches!(
            self.t.get(cursor).map(|input| &input.token.kind),
            Some(TokenKind::Space | TokenKind::Comment)
        ) {
            cursor += 1;
        }
        if !matches!(
            self.t.get(cursor).map(|input| &input.token.kind),
            Some(TokenKind::LBrace)
        ) {
            return false;
        }
        let mut depth = 1usize;
        let mut scan = cursor + 1;
        let close = loop {
            match self.t.get(scan).map(|input| &input.token.kind) {
                Some(TokenKind::LBrace) => depth += 1,
                Some(TokenKind::RBrace) => {
                    depth -= 1;
                    if depth == 0 {
                        break scan;
                    }
                }
                Some(_) => {}
                // Unterminated group: leave it for ordinary recovery rather
                // than guessing where it would have closed.
                None => return false,
            }
            scan += 1;
        };
        if let Some(predicate) = predicate {
            let content = token_text(&self.t[cursor + 1..close]);
            if !predicate(content.trim()) {
                return false;
            }
        }
        self.i = close + 1;
        true
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
        // \newtheorem/\theoremstyle/proof are implemented (see theorems.rs);
        // amsthm takes no package options of its own.
        "amsthm" => options.is_empty(),
        // amsmath/amssymb (math typesetting: \mathbb, \forall, gather,
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

/// The counter style (`a A i I 1`) an enumitem label template selects,
/// mirroring `enumitem_label`'s own template parsing (defaulting to `1`,
/// arabic, exactly like it does). Used by `\setlist{leftmargin=*}` to decide
/// how far its widest-label search needs to look — see `environment`.
fn enumitem_label_style(template: &str) -> char {
    if template.contains('=') {
        let Some(label) = template
            .split(',')
            .find_map(|key| key.trim().strip_prefix("label="))
        else {
            return '1';
        };
        return [
            ("\\alph*", 'a'),
            ("\\Alph*", 'A'),
            ("\\roman*", 'i'),
            ("\\Roman*", 'I'),
        ]
        .iter()
        .find(|(command, _)| label.contains(command))
        .map_or('1', |(_, style)| *style);
    }
    template
        .char_indices()
        .find(|(_, c)| "aAiI1".contains(*c))
        .map_or('1', |(_, style)| style)
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

    /// audit A8: inline math must not gain an inter-word gap the source
    /// never had, on either side of `$...$`.
    #[test]
    fn math_glued_to_following_punctuation_has_no_gap() {
        let (parsed, glued) = items("$x$.");
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let (parsed, spaced) = items("$x$ .");
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let glued_period = glued.iter().find(|i| i.text == ".").unwrap();
        let spaced_period = spaced.iter().find(|i| i.text == ".").unwrap();
        let space = layout::word_space(layout::BODY_SIZE_PT, layout::Font::TimesRoman);
        assert!(
            (spaced_period.x_pt - glued_period.x_pt - space).abs() < 0.01,
            "expected `$x$ .` to sit exactly one word space right of `$x$.`: {} vs {}",
            spaced_period.x_pt,
            glued_period.x_pt
        );
    }

    #[test]
    fn math_followed_by_a_real_space_keeps_exactly_one_word_space() {
        let (parsed, spaced) = items("$x$ y");
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let (parsed, glued) = items("$x$y");
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let spaced_y = spaced.iter().find(|i| i.text == "y").unwrap();
        let glued_y = glued.iter().find(|i| i.text == "y").unwrap();
        let space = layout::word_space(layout::BODY_SIZE_PT, layout::Font::TimesRoman);
        assert!(
            (spaced_y.x_pt - glued_y.x_pt - space).abs() < 0.01,
            "expected `$x$ y` to sit exactly one word space right of `$x$y`: {} vs {}",
            spaced_y.x_pt,
            glued_y.x_pt
        );
    }

    #[test]
    fn text_followed_by_a_real_space_before_math_keeps_exactly_one_word_space() {
        let (parsed, spaced) = items("a $x$");
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let (parsed, glued) = items("a$x$");
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let spaced_x = spaced.iter().find(|i| i.text == "x").unwrap();
        let glued_x = glued.iter().find(|i| i.text == "x").unwrap();
        let space = layout::word_space(layout::BODY_SIZE_PT, layout::Font::TimesRoman);
        assert!(
            (spaced_x.x_pt - glued_x.x_pt - space).abs() < 0.01,
            "expected `a $x$` to sit exactly one word space right of `a$x$`: {} vs {}",
            spaced_x.x_pt,
            glued_x.x_pt
        );
    }

    /// audit A9: a control *word* swallows the whitespace that follows it
    /// (real TeX's "skip blanks" state), so `\normalfont 4` and
    /// `\normalfont4` must typeset identically. Exercised through
    /// `\section{...}` content, the same `inlines_from_tokens` path used by
    /// `\problem`-style macro bodies like `\normalfont[#2 points]`.
    #[test]
    fn normalfont_swallows_its_following_space() {
        let (parsed, spaced) = items("\\section{X\\normalfont 4}");
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let (parsed, glued) = items("\\section{X\\normalfont4}");
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let spaced_four = spaced.iter().find(|i| i.text == "4").unwrap();
        let glued_four = glued.iter().find(|i| i.text == "4").unwrap();
        assert_eq!(
            spaced_four.x_pt, glued_four.x_pt,
            "the space after \\normalfont must not shift what follows it"
        );
    }

    /// `\ ` is a control *symbol* (an escaped literal space), not a control
    /// word, so it is never swallowed; `\\` is the unrelated line-break
    /// token. Neither is affected by the control-word space-swallow rule.
    #[test]
    fn control_space_and_linebreak_are_not_swallowed() {
        let toks = tokenize("x\\ y");
        assert_eq!(toks[0].kind, TokenKind::Word("x".into()));
        assert_eq!(toks[1].kind, TokenKind::Word(" ".into()));
        assert_eq!(toks[2].kind, TokenKind::Word("y".into()));

        let toks = tokenize("x\\\\ y");
        assert_eq!(toks[0].kind, TokenKind::Word("x".into()));
        assert_eq!(toks[1].kind, TokenKind::LineBreak);
        assert_eq!(toks[2].kind, TokenKind::Space);
        assert_eq!(toks[3].kind, TokenKind::Word("y".into()));
    }

    #[test]
    fn tex_input_ligatures_convert_in_ordinary_text() {
        // The exact shape found in fixtures/real-world/hw1/HW1.tex: a ligature
        // pair straddling a word boundary and one embedded inside a single
        // compound word with no surrounding whitespace.
        let source = "``Quoted'' and a turn---after dash, don't stop.\n";
        let (parsed, items) = items(source);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let texts: Vec<&str> = items.iter().map(|item| item.text.as_str()).collect();
        assert!(texts.contains(&"\u{201C}Quoted\u{201D}"), "{texts:?}");
        assert!(texts.contains(&"turn\u{2014}after"), "{texts:?}");
        assert!(texts.iter().any(|t| t.contains('\u{2019}')), "{texts:?}");
    }

    #[test]
    fn tex_input_ligatures_convert_in_headings_and_text_style_arguments() {
        let source = "\\section{Notes---Continued}\n\\textbf{can't---won't}\n";
        let (parsed, items) = items(source);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let texts: Vec<&str> = items.iter().map(|item| item.text.as_str()).collect();
        assert!(texts.iter().any(|t| t.contains('\u{2014}')), "{texts:?}");
        assert!(
            texts.iter().any(|t| t.contains('\u{2019}')),
            "expected a converted apostrophe in {texts:?}"
        );
    }

    #[test]
    fn tex_input_ligatures_never_apply_inside_math() {
        // Math is parsed through an entirely separate path (`math::parse_tokens`)
        // that this function is never wired into; a literal double-hyphen inside
        // `$...$` must stay two separate math minus signs, never an en dash.
        let source = "Text. $a--b$ more text.\n";
        let (parsed, items) = items(source);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert!(!items.iter().any(|item| item.text.contains('\u{2013}')));
        assert!(!items.iter().any(|item| item.text.contains('\u{2014}')));
        assert_eq!(
            items
                .iter()
                .filter(|item| item.text == crate::math::MINUS_SIGN)
                .count(),
            2
        );
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
    fn alignment_declarations_are_group_scoped_and_read_at_paragraph_end() {
        let styles = |source: &str| {
            let parsed = parse(source);
            assert!(
                !parsed
                    .diagnostics
                    .iter()
                    .any(|d| d.message.contains("not supported")
                        || d.message.contains("ragged")
                        || d.message.contains("centering")),
                "{:?}",
                parsed.diagnostics
            );
            parsed
                .blocks
                .iter()
                .map(|block| match block {
                    Block::Styled { style, .. } => Some(*style),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        use ParagraphStyle::{Center, FlushLeft, FlushRight, Quote};
        assert_eq!(
            styles("{\\centering Title\\par} After."),
            [Some(Center), None]
        );
        assert_eq!(
            styles("{\\raggedright Ragged\\par}\n\n{\\raggedleft Left\\par}\n\nPlain."),
            [Some(FlushLeft), Some(FlushRight), None]
        );
        // The group closed before the paragraph ended, so (as in TeX) the
        // declaration no longer applies to it.
        assert_eq!(styles("{\\centering Early} close.\n\nNext."), [None, None]);
        // Scope ends at `\end`; environments that end their paragraph do so
        // with their own declaration still in force.
        assert_eq!(
            styles("\\begin{figure}\\centering Body\\end{figure}\nAfter."),
            [Some(Center), None]
        );
        assert_eq!(
            styles("\\raggedleft\\begin{center}\\RaggedRight Inner\\end{center}\nOuter."),
            [Some(FlushLeft), Some(FlushRight)]
        );
        assert_eq!(
            styles("\\centering\\begin{center}Env\\end{center}\n\\begin{quote}Q\\end{quote}"),
            [Some(Center), Some(Quote)]
        );
        assert_eq!(
            styles("\\documentclass{article}\n\\raggedright\n\\begin{document}\nText.\n\\end{document}"),
            [Some(FlushLeft)]
        );

        // A declared paragraph lays out exactly like its environment form,
        // i.e. it is not justified.
        let words = "Ragged text keeps its natural spaces here. ".repeat(6);
        let positions = |source: String| {
            items(&source)
                .1
                .iter()
                .map(|item| item.x_pt)
                .collect::<Vec<_>>()
        };
        let declared = positions(format!("{{\\raggedright {words}\\par}}"));
        assert_eq!(
            declared,
            positions(format!("\\begin{{flushleft}}{words}\\end{{flushleft}}"))
        );
        assert_ne!(declared, positions(words.clone()));
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

    #[test]
    fn hfill_right_flushes_a_problem_style_subsection_header() {
        // The exact HW1 shape: `\hfill` inside a starred subsection built by
        // a user macro, which routes through `inlines_from_tokens` rather
        // than `command`'s ordinary dispatch.
        let source = r"\newcommand{\problem}[2]{\subsection*{Problem #1 \hfill \normalfont[#2 points]}}\problem{1}{4}";
        let (parsed, items) = items(source);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let problem = items.iter().find(|i| i.text == "Problem").unwrap();
        let points = items.iter().find(|i| i.text == "points]").unwrap();
        assert_eq!(problem.x_pt, layout::MARGIN_PT);
        let points_width = layout::text_width("points]", points.font_size_pt, points.font);
        assert!(
            (points.x_pt + points_width - (layout::PAGE_WIDTH_PT - layout::MARGIN_PT)).abs() < 0.5,
            "expected 'points]' flushed to the right margin, got x_pt={} width={}",
            points.x_pt,
            points_width
        );
    }

    #[test]
    fn multiple_hfills_on_one_line_share_the_leftover_space_equally() {
        let (parsed, items) = items(r"A \hfill B \hfill C");
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let a = items.iter().find(|i| i.text == "A").unwrap();
        let b = items.iter().find(|i| i.text == "B").unwrap();
        let c = items.iter().find(|i| i.text == "C").unwrap();
        assert_eq!(a.x_pt, layout::MARGIN_PT);
        let c_width = layout::text_width("C", c.font_size_pt, c.font);
        assert!(
            (c.x_pt + c_width - (layout::PAGE_WIDTH_PT - layout::MARGIN_PT)).abs() < 0.5,
            "expected the last item flushed to the right margin, got {}",
            c.x_pt
        );
        // Two equal-sized fill gaps: B sits roughly a third of the way across
        // the leftover space, not at the midpoint (one fill) or the margin
        // (no fill).
        let leftover = c.x_pt - a.x_pt;
        assert!(
            (b.x_pt - a.x_pt - leftover / 2.0).abs() < 0.5,
            "expected B roughly midway between A and C, got a={} b={} c={}",
            a.x_pt,
            b.x_pt,
            c.x_pt
        );
    }

    #[test]
    fn hfil_behaves_like_hfill() {
        let (parsed, items) = items(r"A \hfil B");
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let a = items.iter().find(|i| i.text == "A").unwrap();
        let b = items.iter().find(|i| i.text == "B").unwrap();
        let b_width = layout::text_width("B", b.font_size_pt, b.font);
        assert_eq!(a.x_pt, layout::MARGIN_PT);
        assert!((b.x_pt + b_width - (layout::PAGE_WIDTH_PT - layout::MARGIN_PT)).abs() < 0.5);
    }

    #[test]
    fn hspace_inserts_a_fixed_non_stretching_gap() {
        let (parsed, items) = items(r"A\hspace{36pt}B");
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let a = items.iter().find(|i| i.text == "A").unwrap();
        let b = items.iter().find(|i| i.text == "B").unwrap();
        let a_width = layout::text_width("A", a.font_size_pt, a.font);
        // `hspace` (layout.rs) starts from the preceding item's true end
        // (`content_end`), not from the cursor's eagerly reserved trailing
        // inter-word space, so it adds exactly the requested 36pt on top of
        // "A"'s real width — no separate word gap is also added. See the
        // doc comment on `LayoutCursor::hspace`.
        assert!(
            (b.x_pt - (a.x_pt + a_width) - 36.0).abs() < 0.02,
            "a={} a_width={} b={}",
            a.x_pt,
            a_width,
            b.x_pt
        );
    }

    #[test]
    fn hspace_star_and_malformed_dimension_are_handled() {
        let (starred_parsed, starred_items) = items(r"A\hspace*{1em}B");
        assert!(
            starred_parsed.diagnostics.is_empty(),
            "{:?}",
            starred_parsed.diagnostics
        );
        assert!(
            starred_items.iter().any(|i| i.text == "A")
                && starred_items.iter().any(|i| i.text == "B")
        );

        let (malformed_parsed, malformed_items) = items(r"A\hspace{oops}B");
        assert!(malformed_parsed.diagnostics.iter().any(|d| d
            .message
            .contains(r"\hspace requires a recognised dimension")));
        assert!(!malformed_items.iter().any(|i| i.text == "oops"));
    }

    #[test]
    fn unsupported_command_dimension_or_keyword_argument_is_silently_skipped() {
        let (parsed, items) = items(r"Visible \foocmd{0.6em} \barcmd{empty} Tail.");
        assert!(!items.iter().any(|i| i.text == "0.6em"));
        assert!(!items.iter().any(|i| i.text == "empty"));
        assert!(items.iter().any(|i| i.text == "Visible"));
        assert!(items.iter().any(|i| i.text == "Tail."));
        let messages: Vec<&str> = parsed
            .diagnostics
            .iter()
            .map(|d| d.message.as_str())
            .collect();
        assert!(messages.iter().any(|m| m.contains(r"\foocmd")));
        assert!(messages.iter().any(|m| m.contains(r"\barcmd")));
        assert!(parsed.diagnostics.iter().any(|d| d
            .recovery
            .as_deref()
            .is_some_and(|r| r.contains("looked like a parameter"))));
    }

    #[test]
    fn unsupported_command_prose_argument_is_never_swallowed() {
        // Multiple words, and a single capitalized word, both fail the
        // dimension/keyword heuristic and must survive as visible text.
        let (parsed, items) = items(r"\foocmd{Hello world} \barcmd{Capitalized}");
        let _ = parsed;
        assert!(items.iter().any(|i| i.text == "Hello"));
        assert!(items.iter().any(|i| i.text == "world"));
        assert!(items.iter().any(|i| i.text == "Capitalized"));
    }

    #[test]
    fn unsupported_command_single_letter_argument_is_never_swallowed() {
        // A lone lowercase letter is excluded from the keyword heuristic:
        // it is far more likely to be real one-letter content (as in
        // `\def\x{y}`, from crates/compiler/tests/unsupported_inventory.rs)
        // than a parameter like `empty` or `arabic`.
        let (parsed, items) = items(r"\foocmd{y}");
        let _ = parsed;
        assert!(items.iter().any(|i| i.text == "y"));
    }

    #[test]
    fn known_arity_unimplemented_command_always_skips_its_argument() {
        // `1.5` has no unit suffix, so the dimension heuristic alone would
        // never match it: this exercises the explicit
        // `KNOWN_ARITY_UNIMPLEMENTED` list instead.
        let (parsed, items) = items(r"\linespread{1.5} Visible.");
        assert!(!items.iter().any(|i| i.text == "1.5"));
        assert!(items.iter().any(|i| i.text == "Visible."));
        assert!(parsed
            .diagnostics
            .iter()
            .any(|d| d.message.contains(r"\linespread")));
    }

    fn font_of(items: &[crate::layout::TextItem], text: &str) -> layout::Font {
        items
            .iter()
            .find(|item| item.text == text)
            .unwrap_or_else(|| panic!("no item {text:?}"))
            .font
    }

    #[test]
    fn text_style_commands_select_real_core14_variants() {
        use layout::Font;
        let source = r"a \textbf{b $x$ c} \textit{d \textbf{e}} \emph{f \emph{g}} \textsl{h} \texttt{i} \textsf{j} \textbf{\textrm{k}} l";
        let (parsed, items) = items(source);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        for (text, font) in [
            ("a", Font::TimesRoman),
            ("b", Font::TimesBold),
            // Math variables are math italic even inside \textbf.
            ("x", Font::TimesItalic),
            ("c", Font::TimesBold),
            ("d", Font::TimesItalic),
            ("e", Font::TimesBoldItalic),
            ("f", Font::TimesItalic),
            ("g", Font::TimesRoman),
            ("h", Font::TimesItalic),
            ("i", Font::Courier),
            ("j", Font::Helvetica),
            ("k", Font::TimesBold),
            ("l", Font::TimesRoman),
        ] {
            assert_eq!(font_of(&items, text), font, "{text}");
        }
    }

    #[test]
    fn style_declarations_are_scoped_to_groups_and_environments() {
        use layout::Font;
        let source = "\\begin{document}{\\bf a} b {\\it c \\bfseries d} e \\begin{center}\\itshape f\n\ng\\end{center} h {\\ttfamily i \\normalfont j} \\bfseries k\\end{document}";
        let (parsed, items) = items(source);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        for (text, font) in [
            ("a", Font::TimesBold),
            ("b", Font::TimesRoman),
            ("c", Font::TimesItalic),
            ("d", Font::TimesBoldItalic),
            ("e", Font::TimesRoman),
            ("f", Font::TimesItalic),
            ("g", Font::TimesItalic),
            ("h", Font::TimesRoman),
            ("i", Font::Courier),
            ("j", Font::TimesRoman),
            ("k", Font::TimesBold),
        ] {
            assert_eq!(font_of(&items, text), font, "{text}");
        }
    }

    fn size_of(items: &[crate::layout::TextItem], text: &str) -> f64 {
        items
            .iter()
            .find(|item| item.text == text)
            .unwrap_or_else(|| panic!("no item {text:?}"))
            .font_size_pt
    }

    #[test]
    fn size_declarations_scale_relative_to_normalsize() {
        // No `\documentclass`, so the body size defaults to the 12pt class's
        // own table.
        let source = r"\tiny a \scriptsize b \footnotesize c \small d \normalsize e \large f \Large g \LARGE h \huge i \Huge j";
        let (parsed, items) = items(source);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        for (text, size) in [
            ("a", 6.0),
            ("b", 8.0),
            ("c", 10.0),
            ("d", 10.95),
            ("e", 12.0),
            ("f", 14.4),
            ("g", 17.28),
            ("h", 20.74),
            ("i", 24.88),
            ("j", 24.88),
        ] {
            assert_eq!(size_of(&items, text), size, "{text}");
        }
    }

    #[test]
    fn large_scales_text_until_its_group_closes() {
        let (parsed, items) = items(r"Normal {\Large Big text} After");
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert_eq!(size_of(&items, "Normal"), crate::layout::BODY_SIZE_PT);
        assert_eq!(size_of(&items, "Big"), 17.28);
        assert_eq!(size_of(&items, "text"), 17.28);
        assert_eq!(size_of(&items, "After"), crate::layout::BODY_SIZE_PT);
    }

    #[test]
    fn bare_size_declaration_followed_by_a_group_is_not_scoped_to_it() {
        // `\Large{...}` is a common `\textbf{...}`-style misuse: unlike an
        // argument-taking command, `\Large` is a declaration, so it takes
        // effect in whatever scope it appears and stays active past the
        // following group — exactly like real LaTeX, where a group only
        // undoes assignments made *inside* it.
        let (parsed, items) = items(r"\Large{Big} still big");
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert_eq!(size_of(&items, "Big"), 17.28);
        assert_eq!(size_of(&items, "still"), 17.28);
        assert_eq!(size_of(&items, "big"), 17.28);
    }

    #[test]
    fn size_declarations_use_the_active_documentclass_option_table() {
        // The three real LaTeX class tables are not a uniform scale of one
        // another: e.g. `\large` is the same absolute size as `\Large` in
        // the 10pt/11pt classes, but the 12pt class's own
        // `\normalsize`-plus-one-step.
        const LEVELS: [&str; 9] = [
            "tiny",
            "scriptsize",
            "footnotesize",
            "small",
            "large",
            "Large",
            "LARGE",
            "huge",
            "Huge",
        ];
        for (class_option, expected_pt) in [
            (
                "10pt",
                [5.0, 7.0, 8.0, 9.0, 12.0, 14.4, 17.28, 20.74, 24.88],
            ),
            (
                "11pt",
                [6.0, 8.0, 9.0, 10.0, 12.0, 14.4, 17.28, 20.74, 24.88],
            ),
            (
                "12pt",
                [6.0, 8.0, 10.0, 10.95, 14.4, 17.28, 20.74, 24.88, 24.88],
            ),
        ] {
            let body: String = LEVELS
                .iter()
                .enumerate()
                .map(|(i, level)| format!("\\{level} w{i} "))
                .collect();
            let source = format!(
                "\\documentclass[{class_option}]{{article}}\\begin{{document}}{body}\\end{{document}}"
            );
            let parsed = parse(&source);
            assert!(
                parsed.diagnostics.is_empty(),
                "{class_option}: {:?}",
                parsed.diagnostics
            );
            let output =
                crate::incremental::compile_full(&source, layout::LayoutConstraints::default());
            for (i, level) in LEVELS.iter().enumerate() {
                let word = format!("w{i}");
                let size = output
                    .pages
                    .iter()
                    .flat_map(|page| &page.items)
                    .find(|item| item.text == word)
                    .unwrap_or_else(|| panic!("{class_option} \\{level}: no item {word:?}"))
                    .font_size_pt;
                assert_eq!(size, expected_pt[i], "{class_option} \\{level}");
            }
        }
    }

    #[test]
    fn normalsize_is_exactly_the_documentclass_body_size_even_at_11pt() {
        // This compiler's `\documentclass[11pt]` body size is a literal
        // 11pt, not real LaTeX's 10.95pt `\normalsize` (see `class_size_pt`).
        // `\normalsize` must match that approximation exactly, not the real
        // class table value, so text with no size declaration in effect
        // renders identically to before this feature existed.
        let source = r"\documentclass[11pt]{article}\begin{document}Body {\small Small} \normalsize Reset\end{document}";
        let parsed = parse(source);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let output = crate::incremental::compile_full(source, layout::LayoutConstraints::default());
        let size_of = |text: &str| {
            output
                .pages
                .iter()
                .flat_map(|page| &page.items)
                .find(|item| item.text == text)
                .unwrap_or_else(|| panic!("no item {text:?}"))
                .font_size_pt
        };
        assert_eq!(size_of("Body"), 11.0);
        assert_eq!(size_of("Small"), 10.0);
        assert_eq!(size_of("Reset"), 11.0);
    }

    #[test]
    fn size_declarations_grow_the_line_height_so_larger_lines_do_not_overlap() {
        let (parsed, items) = items(r"{\Large Big}\\Small line");
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let big = items.iter().find(|i| i.text == "Big").unwrap();
        let small = items.iter().find(|i| i.text == "Small").unwrap();
        assert!(small.baseline_y_pt > big.baseline_y_pt);
        let gap = small.baseline_y_pt - big.baseline_y_pt;
        // Without this feature `\Large` renders at the plain body size, so
        // the two lines would sit exactly `BODY_SIZE_PT * LINE_SPACING` apart
        // (14.4pt) regardless of the declared size. The real `\Large` line is
        // taller, so the gap to the next line must be strictly larger than
        // that, or the two lines would overlap.
        let old_buggy_gap = crate::layout::BODY_SIZE_PT * crate::layout::LINE_SPACING;
        assert!(
            gap > old_buggy_gap,
            "gap = {gap}, old_buggy_gap = {old_buggy_gap}"
        );
    }

    #[test]
    fn heading_styles_start_bold_and_honour_normalfont() {
        use layout::Font;
        let source = r"\newcommand{\problem}[2]{\subsection*{Problem #1 \normalfont[#2 \textit{pts}]}}\problem{1}{4}";
        let (parsed, items) = items(source);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert_eq!(font_of(&items, "Problem"), Font::TimesBold);
        assert_eq!(font_of(&items, "["), Font::TimesRoman);
        assert_eq!(font_of(&items, "4"), Font::TimesRoman);
        assert_eq!(font_of(&items, "pts"), Font::TimesItalic);
    }

    #[test]
    fn unsupported_parameter_like_argument_is_skipped_but_prose_is_preserved() {
        let source = r"A \unknown{0.6em} B \typo{Readable prose} C";
        let (parsed, items) = items(source);
        let text: Vec<_> = items.iter().map(|item| item.text.as_str()).collect();
        assert!(!text.contains(&"0.6em"), "{text:?}");
        assert!(
            text.contains(&"Readable") && text.contains(&"prose"),
            "{text:?}"
        );
        assert_eq!(
            parsed
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.message.contains("not supported"))
                .count(),
            2
        );
        assert!(parsed.diagnostics.iter().any(|diagnostic| {
            diagnostic.message.contains("\\unknown")
                && diagnostic
                    .recovery
                    .as_deref()
                    .is_some_and(|note| note.contains("parameter"))
        }));
        assert!(parsed.diagnostics.iter().any(|diagnostic| {
            diagnostic.message.contains("\\typo")
                && diagnostic
                    .recovery
                    .as_deref()
                    .is_some_and(|note| note.contains("plain text"))
        }));
    }

    #[test]
    fn malformed_hspace_is_diagnosed_without_leaking_its_argument() {
        let source = r"A\hspace{wide}B";
        let (parsed, items) = items(source);
        assert!(parsed
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("\\hspace requires")));
        assert_eq!(
            items
                .iter()
                .map(|item| item.text.as_str())
                .collect::<Vec<_>>(),
            ["A", "B"]
        );
    }
}
