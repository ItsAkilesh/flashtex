//! Floats (`figure`, `table` and their starred forms), numbered `\caption`,
//! and `\includegraphics` option parsing.
//!
//! A float's body is parsed into its own block list and wrapped in one
//! `Block::Float`; placement (here/top/bottom/float page, deferral) happens in
//! `layout::figures`. Figure and table counters are deliberately local and
//! simple (`P::figure_counter`/`P::table_counter`) so the cross-reference
//! lane (#61) can fold them into a shared counter table later.
//!
//! Honest boundary for graphics: a compile request carries text documents
//! only, so image bytes — and therefore an image's natural size — are never
//! available here. `\includegraphics` becomes a draft-mode frame (graphicx's
//! own `draft` rendering) sized from `width`/`height`/`scale` alone, with one
//! warning per use saying so. No image content is ever invented.

use super::{token_text, Block, Inline, MacroDependency, ParagraphStyle, TextStyle, P};
use crate::diagnostics::Diagnostic;
use crate::lexer::TokenKind;
use crate::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatKind {
    Figure,
    Table,
}

impl FloatKind {
    pub fn from_environment(name: &str) -> Option<Self> {
        match name {
            "figure" | "figure*" => Some(FloatKind::Figure),
            "table" | "table*" => Some(FloatKind::Table),
            _ => None,
        }
    }

    /// article.cls `\figurename`/`\tablename`.
    pub fn caption_name(self) -> &'static str {
        match self {
            FloatKind::Figure => "Figure",
            FloatKind::Table => "Table",
        }
    }
}

/// A float placement specifier after LaTeX's own normalisation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FloatPlacement {
    pub here: bool,
    pub top: bool,
    pub bottom: bool,
    pub page: bool,
    /// `!`: ignore the number and fraction limits for text pages.
    pub force: bool,
    /// `H` from the `float` package: not a float at all, set in the text flow.
    pub here_definitely: bool,
}

impl FloatPlacement {
    /// article.cls `\fps@figure` and `\fps@table`: `tbp`.
    pub const DEFAULT: FloatPlacement = FloatPlacement {
        here: false,
        top: true,
        bottom: true,
        page: true,
        force: false,
        here_definitely: false,
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct Float {
    pub kind: FloatKind,
    pub placement: FloatPlacement,
    /// The `\begin{...}` command and its argument.
    pub span: Span,
    pub body: Vec<Block>,
}

/// An `\includegraphics` length, resolved against the layout at use time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Extent {
    Pt(f64),
    /// A multiple of `\textwidth` (also `\columnwidth`, `\hsize` in one column).
    TextWidth(f64),
    /// A multiple of `\linewidth` (the current, possibly indented, measure).
    LineWidth(f64),
    TextHeight(f64),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GraphicSize {
    pub width: Option<Extent>,
    pub height: Option<Extent>,
    pub scale: f64,
    pub keep_aspect_ratio: bool,
}

/// One open float environment while its body is being parsed.
#[derive(Debug)]
pub(super) struct FloatFrame {
    kind: FloatKind,
    placement: FloatPlacement,
    span: Span,
    /// `env_stack` depth before this float's own entry was pushed.
    env_depth: usize,
    block_start: usize,
    dependency_start: usize,
    /// `\@parboxrestore`: a float body starts outside any list or
    /// paragraph-shape environment; the enclosing ones resume after it.
    outer_styles: Vec<ParagraphStyle>,
    outer_lists: Vec<(String, u32, Option<String>, super::ListSpacing)>,
}

impl P<'_> {
    /// `\begin{figure}` and friends, called before the environment is pushed.
    pub(super) fn begin_float(
        &mut self,
        environment: &str,
        span: Span,
        blocks: &mut Vec<Block>,
        para: &mut Vec<Inline>,
    ) {
        self.flush_paragraph(blocks, para);
        let options = self.float_options();
        let Some(kind) = FloatKind::from_environment(environment) else {
            return;
        };
        if !self.float_frames.is_empty() {
            self.diags.push(Diagnostic::error(
                format!("\\begin{{{environment}}} inside another float: not in outer par mode"),
                Some(span),
                Some("typeset the inner float's body in place inside the outer float".into()),
            ));
            return;
        }
        let placement = match options {
            Some((spec, spec_span)) => self.float_placement(&spec, spec_span),
            None => FloatPlacement::DEFAULT,
        };
        // Float placement depends on what precedes it on the page and defers
        // across later blocks, so cached block fragments cannot be reused.
        self.document_global_state = true;
        self.float_frames.push(FloatFrame {
            kind,
            placement,
            span,
            env_depth: self.env_stack.len(),
            block_start: blocks.len(),
            dependency_start: self.block_dependencies.len(),
            outer_styles: std::mem::take(&mut self.paragraph_styles),
            outer_lists: std::mem::take(&mut self.list_stack),
        });
    }

    /// `\end{figure}` and friends, called after the environment was popped.
    pub(super) fn end_float(&mut self, blocks: &mut Vec<Block>, para: &mut Vec<Inline>) {
        self.flush_paragraph(blocks, para);
        if self
            .float_frames
            .last()
            .is_none_or(|frame| frame.env_depth != self.env_stack.len())
        {
            return;
        }
        let frame = self.float_frames.pop().expect("checked above");
        self.paragraph_styles = frame.outer_styles;
        self.list_stack = frame.outer_lists;
        let body: Vec<Block> = blocks
            .drain(frame.block_start.min(blocks.len())..)
            .collect();
        let mut dependencies: Vec<MacroDependency> = Vec::new();
        let start = frame.dependency_start.min(self.block_dependencies.len());
        for dependency in self.block_dependencies.drain(start..).flatten() {
            if !dependencies.contains(&dependency) {
                dependencies.push(dependency);
            }
        }
        blocks.push(Block::Float(Float {
            kind: frame.kind,
            placement: frame.placement,
            span: frame.span,
            body,
        }));
        self.block_dependencies.push(dependencies);
    }

    /// The `[spec]` after `\begin{figure}`. Brackets are word characters to
    /// the lexer, so `[ht]Body` arrives as one word: split the body text off
    /// and keep it, as `skip_line_break_length` does for `\\[3pt]Next`.
    fn float_options(&mut self) -> Option<(String, Span)> {
        self.skip_spaces();
        let input = self.t.get_mut(self.i)?;
        if let TokenKind::Word(word) = &input.token.kind {
            if let (true, Some(close)) = (word.starts_with('['), word.find(']')) {
                if close + 1 < word.len() {
                    let spec = word[1..close].to_string();
                    let rest = word[close + 1..].to_string();
                    let span = input.token.span;
                    let exact = span.end - span.start == word.len();
                    let spec_span = if exact {
                        Span::in_document(span.document, span.start, span.start + close + 1)
                    } else {
                        span
                    };
                    if exact {
                        input.token.span =
                            Span::in_document(span.document, span.start + close + 1, span.end);
                    }
                    input.token.kind = TokenKind::Word(rest);
                    return Some((spec, spec_span));
                }
            }
        }
        self.optional_bracket_argument()
    }

    /// LaTeX's `\@xfloat` specifier handling: `htbp!` are valid, `H` only
    /// with the `float` package, a lone `h` becomes `ht` with a warning, and
    /// a specifier with no position letters falls back to the default.
    fn float_placement(&mut self, spec: &str, spec_span: Span) -> FloatPlacement {
        let mut placement = FloatPlacement {
            here: false,
            top: false,
            bottom: false,
            page: false,
            force: false,
            here_definitely: false,
        };
        for letter in spec.chars().filter(|ch| !ch.is_whitespace()) {
            match letter {
                'h' => placement.here = true,
                't' => placement.top = true,
                'b' => placement.bottom = true,
                'p' => placement.page = true,
                '!' => placement.force = true,
                'H' if self.packages.iter().any(|package| package == "float") => {
                    placement.here_definitely = true
                }
                other => self.diags.push(Diagnostic::error(
                    if other == 'H' {
                        "unknown float option `H'; it requires \\usepackage{float}".to_string()
                    } else {
                        format!("unknown float option `{other}'")
                    },
                    Some(spec_span),
                    Some("ignored the option".into()),
                )),
            }
        }
        if placement.here_definitely {
            return placement;
        }
        if !(placement.here || placement.top || placement.bottom || placement.page) {
            return FloatPlacement {
                force: placement.force,
                ..FloatPlacement::DEFAULT
            };
        }
        if placement.here && !(placement.top || placement.bottom || placement.page) {
            self.diags.push(Diagnostic::warning(
                "`h' float specifier changed to `ht'",
                Some(spec_span),
                Some("allowed top-of-page placement as LaTeX does".into()),
            ));
            placement.top = true;
        }
        placement
    }

    /// `\caption[short]{long}` inside a float: "Figure n: long". The short
    /// form only feeds lists of figures, which are not implemented.
    pub(super) fn caption(&mut self, span: Span, blocks: &mut Vec<Block>, para: &mut Vec<Inline>) {
        let _short = self.optional_bracket_argument();
        let (tokens, _) = self.required_group("caption", span);
        let Some(kind) = self.float_frames.last().map(|frame| frame.kind) else {
            self.diags.push(Diagnostic::error(
                "\\caption outside float: only supported inside a figure or table environment",
                Some(span),
                Some("typeset the caption text as an ordinary paragraph".into()),
            ));
            let style = self.style;
            para.extend(self.inlines_from_tokens(tokens, style));
            return;
        };
        self.flush_paragraph(blocks, para);
        let number = match kind {
            FloatKind::Figure => {
                self.figure_counter += 1;
                self.figure_counter
            }
            FloatKind::Table => {
                self.table_counter += 1;
                self.table_counter
            }
        };
        self.current_counter = Some(number.to_string());
        let mut content = vec![Inline::Text {
            text: format!("{} {}:", kind.caption_name(), number),
            span,
            style: TextStyle::default(),
            space_before: true,
        }];
        content.extend(self.inlines_from_tokens(tokens, TextStyle::default()));
        blocks.push(Block::FigureCaption { content });
        self.finish_block_dependencies();
    }

    /// `\centering` inside a float: centre the paragraphs that end before
    /// the float closes (the float's end restores the outer styles).
    pub(super) fn float_centering(&mut self) -> bool {
        if self.float_frames.is_empty() {
            return false;
        }
        self.paragraph_styles.push(ParagraphStyle::Center);
        true
    }

    /// `\includegraphics[options]{file}`; the command token is already consumed.
    pub(super) fn include_graphics(&mut self, span: Span, para: &mut Vec<Inline>) {
        let space_before = self.space_precedes(self.i.saturating_sub(1));
        let _starred = self.take_optional_star();
        let options = self
            .optional_bracket_argument()
            .map(|(options, _)| options)
            .unwrap_or_default();
        let (tokens, argument_span) = self.required_group("includegraphics", span);
        let full = span.merge(argument_span);
        let file = self
            .documents
            .get(argument_span.document.0)
            .and_then(|document| document.text.get(argument_span.start..argument_span.end))
            .and_then(|raw| raw.strip_prefix('{')?.strip_suffix('}'))
            .map_or_else(|| token_text(&tokens), str::to_string)
            .trim()
            .to_string();
        if file.is_empty() {
            self.diags.push(Diagnostic::error(
                "\\includegraphics requires a file name",
                Some(full),
                Some("omitted the image and continued".into()),
            ));
            return;
        }
        let (size, ignored) = graphic_size(&options);
        let mut message = format!(
            "\\includegraphics{{{file}}}: image bytes are not available to the compiler \
             (compile requests carry text documents only), so the natural size is unknown"
        );
        if !ignored.is_empty() {
            message.push_str(&format!("; ignored options: {}", ignored.join(", ")));
        }
        self.diags.push(Diagnostic::warning(
            message,
            Some(full),
            Some("drew a draft-style frame sized from the width/height/scale options only".into()),
        ));
        para.push(Inline::Graphic {
            file,
            size,
            span: full,
            space_before,
        });
    }
}

/// graphicx keys this compiler can honour without the image itself. The
/// second value lists every option that was not applied.
pub(crate) fn graphic_size(options: &str) -> (GraphicSize, Vec<String>) {
    let mut size = GraphicSize {
        width: None,
        height: None,
        scale: 1.0,
        keep_aspect_ratio: false,
    };
    let mut ignored = Vec::new();
    for option in options.split(',').map(str::trim).filter(|o| !o.is_empty()) {
        let (key, value) = match option.split_once('=') {
            Some((key, value)) => (key.trim(), Some(value.trim())),
            None => (option, None),
        };
        let applied = match (key, value) {
            ("width", Some(value)) => extent(value).map(|e| size.width = Some(e)),
            ("height" | "totalheight", Some(value)) => extent(value).map(|e| size.height = Some(e)),
            ("scale", Some(value)) => value.parse::<f64>().ok().map(|s| size.scale = s),
            ("keepaspectratio", None | Some("true" | "false")) => {
                size.keep_aspect_ratio = value != Some("false");
                Some(())
            }
            // Draft rendering is the only rendering available either way.
            ("draft" | "final", None | Some("true" | "false")) => Some(()),
            _ => None,
        };
        if applied.is_none() {
            ignored.push(option.to_string());
        }
    }
    (size, ignored)
}

/// `0.8\textwidth`, `\linewidth`, `5cm`, ...
fn extent(value: &str) -> Option<Extent> {
    let value = value.trim();
    for (name, make) in [
        ("\\textwidth", Extent::TextWidth as fn(f64) -> Extent),
        ("\\columnwidth", Extent::TextWidth),
        ("\\hsize", Extent::TextWidth),
        ("\\linewidth", Extent::LineWidth),
        ("\\textheight", Extent::TextHeight),
    ] {
        if let Some(factor) = value.strip_suffix(name) {
            let factor = factor.trim();
            return if factor.is_empty() {
                Some(make(1.0))
            } else {
                factor.parse::<f64>().ok().map(make)
            };
        }
    }
    super::parse_dimen_pt(value).map(Extent::Pt)
}
