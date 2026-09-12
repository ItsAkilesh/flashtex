//! Construction and layout of the deliberately small supported math subset.
//!
//! The constants below approximate classic TeX proportions. TeX normally obtains
//! script shifts, axis height, and fraction gaps from font parameters; FlashTeX
//! does not yet read a real math font and these values are honest approximations.

use crate::diagnostics::Diagnostic;
use crate::lexer::{Token, TokenKind};
use crate::Span;

pub const SCRIPT_SCALE: f64 = 0.7;
pub const SECOND_ORDER_SCRIPT_SCALE: f64 = 0.5;
pub const SUPERSCRIPT_RAISE_EM: f64 = 0.45;
pub const SUBSCRIPT_LOWER_EM: f64 = 0.2;
pub const MATH_AXIS_EM: f64 = 0.25;
pub const FRACTION_GAP_EM: f64 = 0.16;
pub const FRACTION_RULE_EM: f64 = 0.06;
pub const MATRIX_COLUMN_GAP_EM: f64 = 1.0;
pub const MATRIX_ROW_GAP_EM: f64 = 0.3;
pub const QUAD_EM: f64 = 1.0;

#[derive(Debug, Clone, PartialEq)]
pub struct MathList {
    pub atoms: Vec<MathAtom>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MathAtom {
    pub nucleus: Nucleus,
    pub span: Span,
    pub superscript: Option<MathList>,
    pub subscript: Option<MathList>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Nucleus {
    Symbol(String),
    /// Literal text with explicit Roman intent, distinct from math symbols.
    Text(String),
    /// Explicit TeX math glue, measured in ems of the current math style.
    Space {
        em: f64,
    },
    Fraction {
        numerator: MathList,
        denominator: MathList,
    },
    Radical(MathList),
    /// `array`, `cases` and the amsmath matrix environments: a grid of cells
    /// with per-column alignment (`l`, `c`, `r`) and optional stretched fences.
    Matrix {
        rows: Vec<Vec<MathList>>,
        columns: String,
        left: String,
        right: String,
    },
}

/// Math-mode environments implemented as grids: (name, default column
/// alignment repeated for every column, left fence, right fence).
const GRID_ENVIRONMENTS: &[(&str, char, &str, &str)] = &[
    ("array", 'c', "", ""),
    ("matrix", 'c', "", ""),
    ("smallmatrix", 'c', "", ""),
    ("pmatrix", 'c', "(", ")"),
    ("bmatrix", 'c', "[", "]"),
    ("Bmatrix", 'c', "{", "}"),
    ("vmatrix", 'c', "|", "|"),
    ("Vmatrix", 'c', "‖", "‖"),
    ("cases", 'l', "{", ""),
    ("aligned", 'c', "", ""),
    ("gathered", 'c', "", ""),
];

#[derive(Debug, Clone, PartialEq)]
pub struct MathItem {
    /// Explicit font for text nuclei; None retains symbol-driven selection.
    pub font: Option<crate::layout::Font>,
    pub text: String,
    pub x: f64,
    /// Offset from the surrounding text baseline; positive is downward.
    pub baseline: f64,
    pub size: f64,
    pub span: Span,
    /// A real rectangular rule represented alongside the legacy text fallback.
    /// Coordinates are relative to the surrounding math baseline.
    pub rule: Option<MathRule>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MathRule {
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MathBox {
    pub items: Vec<MathItem>,
    pub width: f64,
    pub ascent: f64,
    pub descent: f64,
}

pub fn parse_tokens(tokens: &[Token], diagnostics: &mut Vec<Diagnostic>) -> MathList {
    let split = split_word_tokens(tokens);
    MathParser {
        tokens: &split,
        i: 0,
        depth: 0,
        diagnostics,
    }
    .list(false)
}

/// Maximum nesting of braced math groups, scripts, fractions and radicals.
///
/// The list parser is recursive descent, so a document full of unclosed openers
/// recurses once per opener. Without a bound, a pathological file — or a
/// half-typed one — overflows the stack and kills the worker mid-keystroke.
/// Exceeding the bound is an explicit diagnostic, not a crash.
pub const MAX_MATH_DEPTH: usize = 256;

struct MathParser<'a> {
    tokens: &'a [Token],
    i: usize,
    depth: usize,
    diagnostics: &'a mut Vec<Diagnostic>,
}

impl MathParser<'_> {
    fn list(&mut self, stop_at_brace: bool) -> MathList {
        if self.depth >= MAX_MATH_DEPTH {
            // Consume the rest so the caller cannot loop on the same tokens.
            let span = self.tokens.get(self.i).map(|t| t.span);
            self.diagnostics.push(Diagnostic::error(
                format!("math nesting deeper than {MAX_MATH_DEPTH} levels is not supported"),
                span,
                Some("stopped descending and typeset nothing further in this expression".into()),
            ));
            self.i = self.tokens.len();
            return MathList { atoms: Vec::new() };
        }
        self.depth += 1;
        let result = self.list_inner(stop_at_brace);
        self.depth -= 1;
        result
    }

    fn list_inner(&mut self, stop_at_brace: bool) -> MathList {
        let mut atoms: Vec<MathAtom> = Vec::new();
        while self.i < self.tokens.len() {
            let token = self.tokens[self.i].clone();
            match token.kind {
                TokenKind::Space | TokenKind::ParBreak | TokenKind::Comment => self.i += 1,
                TokenKind::RBrace if stop_at_brace => {
                    self.i += 1;
                    return MathList { atoms };
                }
                TokenKind::RBrace => {
                    self.i += 1;
                    self.diagnostics.push(Diagnostic::error(
                        "unmatched '}' in math mode",
                        Some(token.span),
                        Some("ignored the stray brace and continued".into()),
                    ));
                }
                TokenKind::LBrace => {
                    self.i += 1;
                    atoms.extend(self.list(true).atoms);
                }
                // A bare `&` reaches here only outside a tabular alignment
                // context: `grid_environment` (matrices, `cases`, `array`, …)
                // and the parser's `align`/`gather` row-splitting both consume
                // their own `&` tokens before ever calling into this list, so
                // one seen here is always misplaced.
                TokenKind::Word(ref word) if word == "&" => {
                    self.i += 1;
                    self.diagnostics.push(Diagnostic::error(
                        "misplaced alignment tab character &",
                        Some(token.span),
                        Some("ignored the stray alignment tab and continued".into()),
                    ));
                }
                TokenKind::Superscript | TokenKind::Subscript => {
                    self.i += 1;
                    let script = self.script_argument(token.span);
                    if let Some(atom) = atoms.last_mut() {
                        let slot = if token.kind == TokenKind::Superscript {
                            &mut atom.superscript
                        } else {
                            &mut atom.subscript
                        };
                        if slot.replace(script).is_some() {
                            self.diagnostics.push(Diagnostic::error(
                                "duplicate script on a math atom",
                                Some(token.span),
                                Some("used the last script and continued".into()),
                            ));
                        }
                    } else {
                        self.diagnostics.push(Diagnostic::error(
                            "script marker has no preceding math atom",
                            Some(token.span),
                            Some("ignored the unattached script".into()),
                        ));
                    }
                }
                _ => {
                    if let Some(atom) = self.atom() {
                        atoms.push(atom);
                    }
                }
            }
        }
        if stop_at_brace {
            let span = self.tokens.last().map(|t| t.span);
            self.diagnostics.push(Diagnostic::error(
                "math group is missing its closing brace",
                span,
                Some("closed the group at the math delimiter".into()),
            ));
        }
        MathList { atoms }
    }

    fn script_argument(&mut self, marker: Span) -> MathList {
        while matches!(
            self.tokens.get(self.i).map(|t| &t.kind),
            Some(TokenKind::Space)
        ) {
            self.i += 1;
        }
        if matches!(
            self.tokens.get(self.i).map(|t| &t.kind),
            Some(TokenKind::LBrace)
        ) {
            self.i += 1;
            return self.list(true);
        }
        if let Some(atom) = self.atom() {
            MathList { atoms: vec![atom] }
        } else {
            self.diagnostics.push(Diagnostic::error(
                "math script is missing its argument",
                Some(marker),
                Some("used an empty script and continued".into()),
            ));
            MathList { atoms: Vec::new() }
        }
    }

    fn atom(&mut self) -> Option<MathAtom> {
        let token = self.tokens.get(self.i)?.clone();
        self.i += 1;
        match token.kind {
            TokenKind::Word(word) => {
                let mut chars = word.char_indices();
                let (offset, ch) = chars.next()?;
                let end = offset + ch.len_utf8();
                let span = if token.span.end - token.span.start == word.len() {
                    Span::in_document(
                        token.span.document,
                        token.span.start + offset,
                        token.span.start + end,
                    )
                } else {
                    token.span
                };
                // Word runs are split before parsing so a script attaches to
                // one ordinary atom rather than the entire lexer token.
                debug_assert_eq!(
                    end,
                    word.len(),
                    "word tokens must be split before math parsing"
                );
                // The lexer turns the control symbols `\,` `\:` `\;` into a
                // one-character word spanning two source bytes; in math they are
                // thin/medium/thick spaces (3, 4 and 5 mu), not punctuation.
                if token.span.end - token.span.start == 2 {
                    let mu = match ch {
                        ',' => 3.0,
                        ':' => 4.0,
                        ';' => 5.0,
                        _ => 0.0,
                    };
                    if mu > 0.0 {
                        return Some(space(mu / 18.0, token.span));
                    }
                }
                Some(symbol(ch.to_string(), span))
            }
            TokenKind::Command(name) => Some(self.command_atom(name, token.span)),
            TokenKind::DisplayMathOpen | TokenKind::DisplayMathClose | TokenKind::MathShift => {
                self.diagnostics.push(Diagnostic::error(
                    "unexpected math delimiter inside math mode",
                    Some(token.span),
                    Some("typeset the delimiter literally and continued".into()),
                ));
                Some(symbol("$".into(), token.span))
            }
            TokenKind::LineBreak => Some(symbol("\\\\".into(), token.span)),
            TokenKind::LBrace => Some(symbol("{".into(), token.span)),
            TokenKind::RBrace
            | TokenKind::Space
            | TokenKind::ParBreak
            | TokenKind::Comment
            | TokenKind::Superscript
            | TokenKind::Subscript => None,
        }
    }

    fn command_atom(&mut self, name: String, span: Span) -> MathAtom {
        match name.as_str() {
            "frac" => {
                let numerator = self.required_group("frac", span);
                let denominator = self.required_group("frac", span);
                MathAtom {
                    nucleus: Nucleus::Fraction {
                        numerator,
                        denominator,
                    },
                    span,
                    superscript: None,
                    subscript: None,
                }
            }
            "begin" => self.grid_environment(span),
            "sqrt" => MathAtom {
                nucleus: Nucleus::Radical(self.required_group("sqrt", span)),
                span,
                superscript: None,
                subscript: None,
            },
            "text" => {
                let (text, argument_span) = self.required_text_group("text", span);
                MathAtom {
                    nucleus: Nucleus::Text(text),
                    span: span.merge(argument_span),
                    superscript: None,
                    subscript: None,
                }
            }
            // Delimiter stretching is not implemented yet. Consume and retain
            // the requested delimiter at ordinary size instead of fabricating a
            // hard-coded parenthesis (which would duplicate the source token).
            "bigl" | "bigr" => self.take_delimiter(&name, span),
            "quad" => space(QUAD_EM, span),
            "qquad" => space(2.0 * QUAD_EM, span),
            _ => match command_glyph(&name) {
                Some(glyph) => symbol(glyph.into(), span),
                None => {
                    self.diagnostics.push(Diagnostic::error(
                        format!("\\{} is not supported in math mode", name),
                        Some(span),
                        Some("typeset the command literally and continued".into()),
                    ));
                    symbol(format!("\\{}", name), span)
                }
            },
        }
    }

    fn take_delimiter(&mut self, command: &str, span: Span) -> MathAtom {
        while matches!(
            self.tokens.get(self.i).map(|t| &t.kind),
            Some(TokenKind::Space)
        ) {
            self.i += 1;
        }
        let Some(token) = self.tokens.get(self.i).cloned() else {
            self.diagnostics.push(Diagnostic::error(
                format!("\\{command} requires a following delimiter"),
                Some(span),
                Some("used an empty delimiter and continued".into()),
            ));
            return symbol(String::new(), span);
        };
        let TokenKind::Word(delimiter) = &token.kind else {
            self.diagnostics.push(Diagnostic::error(
                format!("\\{command} requires a following delimiter"),
                Some(span),
                Some("left the following non-delimiter token to be parsed normally".into()),
            ));
            return symbol(String::new(), span);
        };
        if delimiter.chars().count() != 1 || !"()[]{}|./".contains(delimiter.as_str()) {
            self.diagnostics.push(Diagnostic::error(
                format!("\\{command} does not support delimiter {delimiter:?}"),
                Some(span.merge(token.span)),
                Some("typeset the delimiter at ordinary size and continued".into()),
            ));
        }
        self.i += 1;
        symbol(delimiter.clone(), span.merge(token.span))
    }

    fn required_text_group(&mut self, command: &str, span: Span) -> (String, Span) {
        while matches!(
            self.tokens.get(self.i).map(|t| &t.kind),
            Some(TokenKind::Space)
        ) {
            self.i += 1;
        }
        let Some(open) = self.tokens.get(self.i).cloned() else {
            self.diagnostics.push(Diagnostic::error(
                format!("\\{command} requires a braced text argument"),
                Some(span),
                Some("used an empty argument and continued".into()),
            ));
            return (String::new(), span);
        };
        if open.kind != TokenKind::LBrace {
            self.diagnostics.push(Diagnostic::error(
                format!("\\{command} requires a braced text argument"),
                Some(span),
                Some("used an empty argument and continued".into()),
            ));
            return (String::new(), span);
        }
        self.i += 1;
        let mut depth = 1usize;
        let mut text = String::new();
        let mut end = open.span;
        let mut after_comment = false;
        let mut depth_reported = false;
        while let Some(token) = self.tokens.get(self.i).cloned() {
            self.i += 1;
            end = token.span;
            match token.kind {
                TokenKind::LBrace => {
                    depth += 1;
                    if depth > MAX_MATH_DEPTH && !depth_reported {
                        depth_reported = true;
                        self.diagnostics.push(Diagnostic::error(
                            "text group nesting exceeds the supported math depth",
                            Some(token.span),
                            Some("continued bounded iterative recovery".into()),
                        ));
                    }
                }
                TokenKind::RBrace => {
                    depth -= 1;
                    if depth == 0 {
                        return (text, open.span.merge(end));
                    }
                }
                TokenKind::Word(word) => text.push_str(&word),
                TokenKind::Space => {
                    if !after_comment {
                        text.push(' ');
                    }
                }
                TokenKind::Comment => {
                    after_comment = true;
                    continue;
                }
                TokenKind::ParBreak => {
                    self.diagnostics.push(Diagnostic::error(
                        format!("paragraph breaks are not supported inside \\{command}"),
                        Some(token.span),
                        Some("collapsed the paragraph break to one space and continued".into()),
                    ));
                    text.push(' ');
                }
                TokenKind::LineBreak => {
                    self.diagnostics.push(Diagnostic::error(
                        format!("line breaks are not supported inside \\{command}"),
                        Some(token.span),
                        Some("typeset the line-break command literally and continued".into()),
                    ));
                    text.push_str("\\\\");
                }
                TokenKind::Command(name) => {
                    self.diagnostics.push(Diagnostic::error(
                        format!("\\{name} is not supported inside \\{command}"),
                        Some(token.span),
                        Some("typeset the command name literally and continued".into()),
                    ));
                    text.push('\\');
                    text.push_str(&name);
                }
                TokenKind::MathShift
                | TokenKind::DisplayMathOpen
                | TokenKind::DisplayMathClose
                | TokenKind::Superscript
                | TokenKind::Subscript => {
                    self.diagnostics.push(Diagnostic::error(
                        format!("math syntax is not supported inside \\{command}"),
                        Some(token.span),
                        Some("typeset the token literally and continued".into()),
                    ));
                    text.push_str(match token.kind {
                        TokenKind::MathShift => "$",
                        TokenKind::DisplayMathOpen => "\\[",
                        TokenKind::DisplayMathClose => "\\]",
                        TokenKind::Superscript => "^",
                        TokenKind::Subscript => "_",
                        _ => unreachable!(),
                    });
                }
            }
            after_comment = false;
        }
        self.diagnostics.push(Diagnostic::error(
            format!("argument to \\{command} is missing its closing brace"),
            Some(open.span),
            Some("closed the text argument at the math delimiter".into()),
        ));
        (text, open.span.merge(end))
    }

    /// Reads `{name}` after a `\begin` as plain characters.
    fn environment_name(&mut self) -> Option<String> {
        while matches!(
            self.tokens.get(self.i).map(|t| &t.kind),
            Some(TokenKind::Space)
        ) {
            self.i += 1;
        }
        if !matches!(
            self.tokens.get(self.i).map(|t| &t.kind),
            Some(TokenKind::LBrace)
        ) {
            return None;
        }
        let mut cursor = self.i + 1;
        let mut name = String::new();
        loop {
            match self.tokens.get(cursor).map(|t| &t.kind) {
                Some(TokenKind::Word(ch)) => name.push_str(ch),
                Some(TokenKind::RBrace) => break,
                _ => return None,
            }
            cursor += 1;
        }
        self.i = cursor + 1;
        Some(name)
    }

    /// `\begin{env} cell & cell \\ ... \end{env}` for the grid environments.
    fn grid_environment(&mut self, span: Span) -> MathAtom {
        let unsupported = |name: &str| format!("\\begin{{{name}}} is not supported in math mode");
        let Some(name) = self.environment_name() else {
            self.diagnostics.push(Diagnostic::error(
                "\\begin requires a braced environment name",
                Some(span),
                Some("typeset the command literally and continued".into()),
            ));
            return symbol("\\begin".into(), span);
        };
        let Some(&(_, default_align, left, right)) =
            GRID_ENVIRONMENTS.iter().find(|(env, ..)| *env == name)
        else {
            self.diagnostics.push(Diagnostic::error(
                unsupported(&name),
                Some(span),
                Some("typeset the environment body inline".into()),
            ));
            return symbol(String::new(), span);
        };
        let mut columns = String::new();
        if name == "array" {
            if let Some(TokenKind::LBrace) = self.tokens.get(self.i).map(|t| &t.kind) {
                self.i += 1;
                while let Some(token) = self.tokens.get(self.i) {
                    self.i += 1;
                    match &token.kind {
                        TokenKind::RBrace => break,
                        TokenKind::Word(ch) if matches!(ch.as_str(), "l" | "c" | "r") => {
                            columns.push_str(ch)
                        }
                        _ => {}
                    }
                }
            }
        }
        if name == "aligned" {
            columns = "rl".repeat(8);
        }
        let mut rows: Vec<Vec<Vec<Token>>> = vec![vec![Vec::new()]];
        let mut depth = 0usize;
        let mut nesting = 0usize;
        let mut closed = false;
        while let Some(token) = self.tokens.get(self.i).cloned() {
            self.i += 1;
            let top = depth == 0 && nesting == 0;
            match &token.kind {
                TokenKind::Command(command) if command == "begin" => nesting += 1,
                TokenKind::Command(command) if command == "end" => {
                    if nesting == 0 && depth == 0 {
                        let before = self.i;
                        if self.environment_name().as_deref() == Some(name.as_str()) {
                            closed = true;
                            break;
                        }
                        self.i = before;
                    }
                    nesting = nesting.saturating_sub(1);
                }
                TokenKind::LBrace => depth += 1,
                TokenKind::RBrace => depth = depth.saturating_sub(1),
                _ => {}
            }
            let row = rows.last_mut().expect("at least one row");
            match &token.kind {
                TokenKind::LineBreak if top => {
                    // Skip an optional `[<length>]` row-spacing argument.
                    if matches!(&self.tokens.get(self.i).map(|t| &t.kind), Some(TokenKind::Word(w)) if w == "[")
                    {
                        while let Some(t) = self.tokens.get(self.i) {
                            self.i += 1;
                            if matches!(&t.kind, TokenKind::Word(w) if w == "]") {
                                break;
                            }
                        }
                    }
                    rows.push(vec![Vec::new()]);
                }
                TokenKind::Word(w) if top && w == "&" => row.push(Vec::new()),
                _ => row.last_mut().expect("at least one cell").push(token),
            }
        }
        if !closed {
            self.diagnostics.push(Diagnostic::error(
                format!(
                    "\\begin{{{name}}} has no matching \\end{{{name}}} in this math expression"
                ),
                Some(span),
                Some("closed the environment at the math delimiter".into()),
            ));
        }
        if rows.len() > 1
            && rows.last().is_some_and(|cells| {
                cells.iter().flatten().all(|t| {
                    matches!(
                        t.kind,
                        TokenKind::Space | TokenKind::Comment | TokenKind::ParBreak
                    )
                })
            })
        {
            rows.pop();
        }
        let rows = rows
            .into_iter()
            .map(|cells| {
                cells
                    .into_iter()
                    .map(|cell| {
                        MathParser {
                            tokens: &cell,
                            i: 0,
                            depth: self.depth,
                            diagnostics: self.diagnostics,
                        }
                        .list(false)
                    })
                    .collect()
            })
            .collect::<Vec<Vec<MathList>>>();
        let width = rows.iter().map(Vec::len).max().unwrap_or(0);
        let mut columns: String = columns.chars().take(width).collect();
        while columns.chars().count() < width {
            columns.push(default_align);
        }
        MathAtom {
            nucleus: Nucleus::Matrix {
                rows,
                columns,
                left: left.into(),
                right: right.into(),
            },
            span,
            superscript: None,
            subscript: None,
        }
    }

    fn required_group(&mut self, command: &str, span: Span) -> MathList {
        while matches!(
            self.tokens.get(self.i).map(|t| &t.kind),
            Some(TokenKind::Space)
        ) {
            self.i += 1;
        }
        if matches!(
            self.tokens.get(self.i).map(|t| &t.kind),
            Some(TokenKind::LBrace)
        ) {
            self.i += 1;
            self.list(true)
        } else {
            self.diagnostics.push(Diagnostic::error(
                format!("\\{} requires a braced math argument", command),
                Some(span),
                Some("used an empty argument and continued".into()),
            ));
            MathList { atoms: Vec::new() }
        }
    }
}

fn symbol(text: String, span: Span) -> MathAtom {
    MathAtom {
        nucleus: Nucleus::Symbol(text),
        span,
        superscript: None,
        subscript: None,
    }
}

fn space(em: f64, span: Span) -> MathAtom {
    MathAtom {
        nucleus: Nucleus::Space { em },
        span,
        superscript: None,
        subscript: None,
    }
}

/// Every named symbol the math layer can emit, as (command, rendered glyph).
///
/// The export adapter in `crate::export` is tested against this exact table, so
/// adding a symbol here without giving it an export mapping fails the build's
/// tests rather than silently producing a glyph the PDF path turns into `?`.
pub const COMMAND_GLYPHS: &[(&str, &str)] = &[
    ("alpha", "α"),
    ("beta", "β"),
    ("gamma", "γ"),
    ("delta", "δ"),
    ("theta", "θ"),
    ("lambda", "λ"),
    ("mu", "μ"),
    ("pi", "π"),
    ("sigma", "σ"),
    ("phi", "φ"),
    ("omega", "ω"),
    ("times", "×"),
    ("div", "÷"),
    ("pm", "±"),
    ("leq", "≤"),
    ("geq", "≥"),
    ("neq", "≠"),
    ("approx", "≈"),
    ("cdot", "·"),
    ("infty", "∞"),
    ("sum", "∑"),
    ("int", "∫"),
    ("in", "∈"),
    ("forall", "∀"),
    ("exists", "∃"),
    ("vee", "∨"),
    ("Rightarrow", "⇒"),
    ("mid", "∣"),
    // Symbol.afm has no 0x27F8..0x27FF long-arrow range, only the shorter
    // 0x21D2 double-arrow already used for `\Rightarrow`. Reusing that real
    // glyph loses only the extra stroke length — the same approximation
    // class `take_delimiter` already makes for `\bigl`/`\bigr` (real parens,
    // no size scaling).
    ("Longrightarrow", "⇒"),
];

/// The rule character used to draw fraction bars.
///
/// This is a stand-in, not a real glyph: runtime-v1 has no rule item type yet
/// (see issue #9). No font contains it, so it is deliberately unrepresentable in
/// the export adapter and is reported rather than silently substituted.
pub const FRACTION_RULE_CHAR: char = '\u{2500}';

fn command_glyph(name: &str) -> Option<&'static str> {
    COMMAND_GLYPHS
        .iter()
        .find(|(command, _)| *command == name)
        .map(|(_, glyph)| *glyph)
}

pub fn layout(list: &MathList, size: f64, diagnostics: &mut Vec<Diagnostic>) -> MathBox {
    layout_list(list, size, size, 0, diagnostics)
}

fn layout_list(
    list: &MathList,
    size: f64,
    root_size: f64,
    level: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> MathBox {
    let mut out = MathBox {
        items: Vec::new(),
        width: 0.0,
        ascent: size,
        descent: 0.2 * size,
    };
    for atom in &list.atoms {
        let mut nucleus = layout_nucleus(atom, size, root_size, level, diagnostics);
        let nucleus_width = nucleus.width;
        offset_items(&mut nucleus.items, out.width, 0.0);
        out.ascent = out.ascent.max(nucleus.ascent);
        out.descent = out.descent.max(nucleus.descent);
        out.items.extend(nucleus.items);

        let script_size = if level == 0 {
            root_size * SCRIPT_SCALE
        } else {
            root_size * SECOND_ORDER_SCRIPT_SCALE
        };
        let mut script_width: f64 = 0.0;
        if let Some(sup) = &atom.superscript {
            let mut b = layout_list(sup, script_size, root_size, level + 1, diagnostics);
            let dy = -SUPERSCRIPT_RAISE_EM * size;
            offset_items(&mut b.items, out.width + nucleus_width, dy);
            out.ascent = out.ascent.max(b.ascent - dy);
            script_width = script_width.max(b.width);
            out.items.extend(b.items);
        }
        if let Some(sub) = &atom.subscript {
            let mut b = layout_list(sub, script_size, root_size, level + 1, diagnostics);
            let dy = SUBSCRIPT_LOWER_EM * size;
            offset_items(&mut b.items, out.width + nucleus_width, dy);
            out.descent = out.descent.max(b.descent + dy);
            script_width = script_width.max(b.width);
            out.items.extend(b.items);
        }
        out.width += nucleus_width + script_width;
    }
    out
}

fn layout_nucleus(
    atom: &MathAtom,
    size: f64,
    root_size: f64,
    level: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> MathBox {
    match &atom.nucleus {
        Nucleus::Symbol(text) | Nucleus::Text(text) => MathBox {
            items: vec![MathItem {
                font: matches!(atom.nucleus, Nucleus::Text(_))
                    .then_some(crate::layout::Font::TimesRoman),
                text: text.clone(),
                x: 0.0,
                baseline: 0.0,
                size,
                span: atom.span,
                rule: None,
            }],
            width: crate::layout::shaped_width(
                text,
                size,
                if matches!(atom.nucleus, Nucleus::Text(_)) {
                    crate::layout::Font::TimesRoman
                } else {
                    crate::layout::math_font(text)
                },
                atom.span,
                diagnostics,
            )
            .0,
            ascent: size,
            descent: 0.2 * size,
        },
        Nucleus::Space { em } => MathBox {
            items: Vec::new(),
            width: em * size,
            ascent: size,
            descent: 0.2 * size,
        },
        Nucleus::Radical(body) => {
            let mut b = layout_list(body, size, root_size, level, diagnostics);
            let radical_width = crate::layout::shaped_width(
                "√",
                size,
                crate::layout::math_font("√"),
                atom.span,
                diagnostics,
            )
            .0;
            offset_items(&mut b.items, radical_width, 0.0);
            b.items.insert(
                0,
                MathItem {
                    font: None,
                    text: "√".into(),
                    x: 0.0,
                    baseline: 0.0,
                    size,
                    span: atom.span,
                    rule: None,
                },
            );
            b.width += radical_width;
            b
        }
        Nucleus::Fraction {
            numerator,
            denominator,
        } => {
            let child_size = if level == 0 {
                root_size * SCRIPT_SCALE
            } else {
                root_size * SECOND_ORDER_SCRIPT_SCALE
            };
            let mut num = layout_list(numerator, child_size, root_size, level + 1, diagnostics);
            let mut den = layout_list(denominator, child_size, root_size, level + 1, diagnostics);
            let pad = 0.12 * size;
            let natural_width = num.width.max(den.width) + 2.0 * pad;
            let axis = -MATH_AXIS_EM * size;
            let rule = FRACTION_RULE_EM * size;
            // This legacy string is only a paint fallback. Its geometry is the
            // real rule width and does not pretend U+2500 exists in a Core 14 face.
            let rule_text = "─".to_string();
            let width = natural_width;
            let num_dy = axis - FRACTION_GAP_EM * size - rule / 2.0 - num.descent;
            let den_dy = axis + FRACTION_GAP_EM * size + rule / 2.0 + den.ascent;
            let num_x = (width - num.width) / 2.0;
            let den_x = (width - den.width) / 2.0;
            offset_items(&mut num.items, num_x, num_dy);
            offset_items(&mut den.items, den_x, den_dy);
            let mut items = num.items;
            items.push(MathItem {
                font: None,
                text: rule_text,
                x: 0.0,
                baseline: axis + rule / 2.0,
                size: child_size,
                span: atom.span,
                rule: Some(MathRule {
                    y: axis - rule / 2.0,
                    width,
                    height: rule,
                }),
            });
            items.extend(den.items);
            MathBox {
                items,
                width,
                ascent: (num.ascent - num_dy).max(size * 0.5),
                descent: (den.descent + den_dy).max(size * 0.2),
            }
        }
        Nucleus::Matrix {
            rows,
            columns,
            left,
            right,
        } => layout_matrix(
            atom,
            rows,
            columns,
            (left, right),
            size,
            root_size,
            level,
            diagnostics,
        ),
    }
}

/// Lays out a grid centred on the math axis, with fences scaled to its height.
#[allow(clippy::too_many_arguments)]
fn layout_matrix(
    atom: &MathAtom,
    rows: &[Vec<MathList>],
    columns: &str,
    fences: (&str, &str),
    size: f64,
    root_size: f64,
    level: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> MathBox {
    let boxes: Vec<Vec<MathBox>> = rows
        .iter()
        .map(|row| {
            row.iter()
                .map(|cell| layout_list(cell, size, root_size, level, diagnostics))
                .collect()
        })
        .collect();
    let aligns: Vec<char> = columns.chars().collect();
    let mut widths = vec![0.0f64; aligns.len()];
    for row in &boxes {
        for (column, b) in row.iter().enumerate() {
            widths[column] = widths[column].max(b.width);
        }
    }
    let column_gap = MATRIX_COLUMN_GAP_EM * size;
    let row_gap = MATRIX_ROW_GAP_EM * size;
    // Row baselines relative to the first row's baseline.
    let mut baselines = Vec::with_capacity(boxes.len());
    let mut y = 0.0;
    for (index, row) in boxes.iter().enumerate() {
        let ascent = row.iter().map(|b| b.ascent).fold(size * 0.7, f64::max);
        if index > 0 {
            y += ascent + row_gap;
        }
        baselines.push(y);
        y += row.iter().map(|b| b.descent).fold(size * 0.2, f64::max);
    }
    let first_ascent = boxes.first().map_or(size * 0.7, |row| {
        row.iter().map(|b| b.ascent).fold(size * 0.7, f64::max)
    });
    let height = first_ascent + y;
    // Centre the grid on the math axis.
    let shift = -MATH_AXIS_EM * size - height / 2.0 + first_ascent;
    let (left, right) = fences;
    let fence_size = height.max(size);
    let fence_width = |text: &str, diagnostics: &mut Vec<Diagnostic>| {
        if text.is_empty() {
            0.0
        } else {
            crate::layout::shaped_width(
                text,
                fence_size,
                crate::layout::math_font(text),
                atom.span,
                diagnostics,
            )
            .0
        }
    };
    let left_width = fence_width(left, diagnostics);
    let mut items = Vec::new();
    // A fence glyph's visual centre sits roughly 0.3em above its baseline.
    let fence_baseline = -MATH_AXIS_EM * size + 0.3 * fence_size;
    if !left.is_empty() {
        items.push(MathItem {
            font: None,
            text: left.into(),
            x: 0.0,
            baseline: fence_baseline,
            size: fence_size,
            span: atom.span,
            rule: None,
        });
    }
    let pad = if left.is_empty() { 0.0 } else { 0.15 * size };
    let mut grid_width = 0.0;
    for (row, baseline) in boxes.into_iter().zip(&baselines) {
        let mut x = left_width + pad;
        for (column, mut b) in row.into_iter().enumerate() {
            let dx = match aligns[column] {
                'r' => widths[column] - b.width,
                'c' => (widths[column] - b.width) / 2.0,
                _ => 0.0,
            };
            offset_items(&mut b.items, x + dx, baseline + shift);
            items.extend(b.items);
            x += widths[column] + column_gap;
        }
    }
    if !widths.is_empty() {
        grid_width = widths.iter().sum::<f64>() + column_gap * (widths.len() - 1) as f64;
    }
    let mut width = left_width + pad + grid_width;
    if !right.is_empty() {
        width += 0.15 * size;
        items.push(MathItem {
            font: None,
            text: right.into(),
            x: width,
            baseline: fence_baseline,
            size: fence_size,
            span: atom.span,
            rule: None,
        });
        width += fence_width(right, diagnostics);
    }
    MathBox {
        items,
        width,
        ascent: (first_ascent - shift).max(size),
        descent: (y + shift).max(0.2 * size),
    }
}

fn offset_items(items: &mut [MathItem], dx: f64, dy: f64) {
    for item in items {
        item.x += dx;
        item.baseline += dy;
    }
}

/// Split lexer word runs into one token per Unicode scalar for atom attachment.
fn split_word_tokens(tokens: &[Token]) -> Vec<Token> {
    let mut out = Vec::new();
    for token in tokens {
        if let TokenKind::Word(word) = &token.kind {
            let source_matches_word = token.span.end - token.span.start == word.len();
            for (offset, ch) in word.char_indices() {
                out.push(Token {
                    kind: TokenKind::Word(ch.to_string()),
                    span: if source_matches_word {
                        Span::in_document(
                            token.span.document,
                            token.span.start + offset,
                            token.span.start + offset + ch.len_utf8(),
                        )
                    } else {
                        // Macro replacement text has no byte range of its own.
                        // Preserve the invocation attribution for every atom
                        // instead of fabricating per-glyph provenance.
                        token.span
                    },
                });
            }
        } else {
            out.push(token.clone());
        }
    }
    out
}

/// Shifts every span in a math list by `delta` bytes.
///
/// Incremental reuse moves unchanged blocks when earlier text grows or shrinks.
/// A math list nests — scripts, fractions and radicals each hold their own list —
/// so shifting only the outer span would leave every inner span pointing at the
/// previous revision's bytes, and source navigation would land in the wrong place.
pub fn shift_list(list: &MathList, delta: isize) -> MathList {
    MathList {
        atoms: list.atoms.iter().map(|a| shift_atom(a, delta)).collect(),
    }
}

fn shift_atom(atom: &MathAtom, delta: isize) -> MathAtom {
    MathAtom {
        nucleus: match &atom.nucleus {
            Nucleus::Symbol(s) => Nucleus::Symbol(s.clone()),
            Nucleus::Text(s) => Nucleus::Text(s.clone()),
            Nucleus::Space { em } => Nucleus::Space { em: *em },
            Nucleus::Fraction {
                numerator,
                denominator,
            } => Nucleus::Fraction {
                numerator: shift_list(numerator, delta),
                denominator: shift_list(denominator, delta),
            },
            Nucleus::Radical(inner) => Nucleus::Radical(shift_list(inner, delta)),
            Nucleus::Matrix {
                rows,
                columns,
                left,
                right,
            } => Nucleus::Matrix {
                rows: rows
                    .iter()
                    .map(|row| row.iter().map(|cell| shift_list(cell, delta)).collect())
                    .collect(),
                columns: columns.clone(),
                left: left.clone(),
                right: right.clone(),
            },
        },
        span: shift(atom.span, delta),
        superscript: atom.superscript.as_ref().map(|l| shift_list(l, delta)),
        subscript: atom.subscript.as_ref().map(|l| shift_list(l, delta)),
    }
}

fn shift(span: Span, delta: isize) -> Span {
    let apply = |v: usize| -> usize {
        if delta >= 0 {
            v.saturating_add(delta as usize)
        } else {
            v.saturating_sub(delta.unsigned_abs())
        }
    };
    Span::in_document(span.document, apply(span.start), apply(span.end))
}

#[cfg(test)]
mod parse_tests {
    use super::*;

    #[test]
    fn logical_commands_are_real_exportable_symbol_atoms() {
        let mut diagnostics = Vec::new();
        let tokens = crate::lexer::tokenize(r"\in\forall\exists\vee\Rightarrow\mid");
        let list = parse_tokens(&tokens, &mut diagnostics);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        let glyphs: Vec<&str> = list
            .atoms
            .iter()
            .map(|atom| match &atom.nucleus {
                Nucleus::Symbol(text) => text.as_str(),
                other => panic!("expected symbol, got {other:?}"),
            })
            .collect();
        assert_eq!(glyphs, ["∈", "∀", "∃", "∨", "⇒", "∣"]);
        let _ = layout(&list, 12.0, &mut diagnostics);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn delimiter_sizes_consume_the_source_delimiter_once() {
        let mut diagnostics = Vec::new();
        let tokens = crate::lexer::tokenize(r"\bigl(x\bigr)");
        let list = parse_tokens(&tokens, &mut diagnostics);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        let glyphs: Vec<&str> = list
            .atoms
            .iter()
            .map(|atom| match &atom.nucleus {
                Nucleus::Symbol(text) => text.as_str(),
                other => panic!("expected symbol, got {other:?}"),
            })
            .collect();
        assert_eq!(glyphs, ["(", "x", ")"]);
    }

    #[test]
    fn bare_ampersand_outside_alignment_is_diagnosed() {
        // Reference-corpus negative fixture `error-extra-math-align`: a `&`
        // in ordinary (non-tabular) math used to pass through silently as a
        // literal symbol. `grid_environment` (matrices, `cases`, `array`)
        // and the parser's align/gather row-splitting both consume their own
        // `&` before it ever reaches this list, so one seen here is always a
        // misplaced alignment tab.
        let mut diagnostics = Vec::new();
        let tokens = crate::lexer::tokenize("a & b");
        let list = parse_tokens(&tokens, &mut diagnostics);
        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
        assert!(diagnostics[0].message.contains("misplaced alignment tab"));
        let glyphs: Vec<&str> = list
            .atoms
            .iter()
            .map(|atom| match &atom.nucleus {
                Nucleus::Symbol(text) => text.as_str(),
                other => panic!("expected symbol, got {other:?}"),
            })
            .collect();
        assert_eq!(glyphs, ["a", "b"], "the stray & is dropped, not typeset");
    }

    #[test]
    fn quad_text_and_qquad_have_distinct_semantics() {
        let mut diagnostics = Vec::new();
        let tokens = crate::lexer::tokenize(r"\quad\text{two words}\qquad");
        let list = parse_tokens(&tokens, &mut diagnostics);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert!(matches!(list.atoms[0].nucleus, Nucleus::Space { em } if em == 1.0));
        assert!(matches!(&list.atoms[1].nucleus, Nucleus::Text(text) if text == "two words"));
        assert!(matches!(list.atoms[2].nucleus, Nucleus::Space { em } if em == 2.0));

        let laid_out = layout(&list, 12.0, &mut diagnostics);
        assert_eq!(laid_out.items.len(), 1, "spacing must not emit fake glyphs");
        assert_eq!(
            laid_out.items[0].font,
            Some(crate::layout::Font::TimesRoman),
            "text nuclei must retain explicit Roman intent"
        );
        let text_width =
            crate::layout::text_width("two words", 12.0, crate::layout::Font::TimesRoman);
        assert!((laid_out.width - (text_width + 36.0)).abs() < 0.001);
    }

    #[test]
    fn malformed_delimiter_and_text_arguments_remain_diagnostic() {
        for source in [r"\bigl", r"\text unbraced"] {
            let mut diagnostics = Vec::new();
            let tokens = crate::lexer::tokenize(source);
            let _ = parse_tokens(&tokens, &mut diagnostics);
            assert!(!diagnostics.is_empty(), "{source:?} must remain diagnostic");
        }

        let mut diagnostics = Vec::new();
        let tokens = crate::lexer::tokenize(r"\text x");
        let list = parse_tokens(&tokens, &mut diagnostics);
        assert!(matches!(list.atoms[0].nucleus, Nucleus::Text(ref text) if text.is_empty()));
        assert!(matches!(list.atoms[1].nucleus, Nucleus::Symbol(ref text) if text == "x"));
    }
}

#[cfg(test)]
mod shift_tests {
    use super::*;

    #[test]
    fn shifting_reaches_nested_spans() {
        let mut diagnostics = Vec::new();
        let tokens = crate::lexer::tokenize("\\frac{a^2}{b}");
        let list = parse_tokens(&tokens, &mut diagnostics);
        let shifted = shift_list(&list, 10);

        fn min_start(list: &MathList) -> usize {
            list.atoms
                .iter()
                .map(|a| {
                    let nested = match &a.nucleus {
                        Nucleus::Symbol(_) => usize::MAX,
                        Nucleus::Text(_) => usize::MAX,
                        Nucleus::Space { .. } => usize::MAX,
                        Nucleus::Fraction {
                            numerator,
                            denominator,
                        } => min_start(numerator).min(min_start(denominator)),
                        Nucleus::Radical(inner) => min_start(inner),
                        Nucleus::Matrix { rows, .. } => rows
                            .iter()
                            .flatten()
                            .map(min_start)
                            .min()
                            .unwrap_or(usize::MAX),
                    };
                    let scripts = a
                        .superscript
                        .as_ref()
                        .map(min_start)
                        .unwrap_or(usize::MAX)
                        .min(a.subscript.as_ref().map(min_start).unwrap_or(usize::MAX));
                    a.span.start.min(nested).min(scripts)
                })
                .min()
                .unwrap_or(usize::MAX)
        }

        assert_eq!(min_start(&shifted), min_start(&list) + 10);
    }
}
