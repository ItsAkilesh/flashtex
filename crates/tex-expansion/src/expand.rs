//! The expansion/execution engine: TeX's `get_next` + `expand` + a slice of
//! `main_control` folded into one pass (TeXbook ch. 20 & 23-24). Since
//! FlashTeX's typesetting stage is a separate crate, this engine's output
//! is the fully expanded, fully "assignment-executed" content-token
//! stream: macros are gone, conditionals are resolved, `\def`/`\let`/
//! register assignments have taken effect, and what remains are character
//! tokens plus any control sequences we don't recognize (left untouched
//! for the typesetting layer, e.g. `\section`, `\hskip`, font commands).

use std::collections::HashMap;
use std::rc::Rc;

use crate::catcode::CatCode;
use crate::conditionals::{ConditionalStack, IfBranch, IfShape};
use crate::error::{Diagnostic, Limits};
use crate::lexer::Lexer;
use crate::macro_def::{BodyPart, MacroDef, MacroFlags, ParamPart};
use crate::registers::{absolute_unit_sp_per_unit, scale_decimal, DefaultFontMetrics, FontMetrics, Glue};
use crate::scopes::{Meaning, Primitive, RegisterKind, Scopes};
use crate::span::Span;
use crate::token::{Token, TokenKind};

struct Pending {
    tok: Token,
    frozen: bool,
}

enum Input<'a> {
    Text(Lexer<'a>),
    Toks(Vec<Pending>, usize),
}

pub struct Engine<'a> {
    sources: Vec<Input<'a>>,
    next_source_id: u32,
    scopes: Scopes,
    conditionals: ConditionalStack,
    limits: Limits,
    steps: u64,
    diagnostics: Vec<Diagnostic>,
    pending_global: bool,
    pending_long: bool,
    pending_outer: bool,
    metrics: Box<dyn FontMetrics>,
    /// Rough mode input for `\ifvmode`/`\ifhmode`/`\ifmmode`/`\ifinner`,
    /// since we have no real typesetter yet; the host can override via
    /// `set_mode` before expanding a fragment (see CONTRACT.md).
    mode: Mode,
    /// Names of currently-open `\begin{...}` environments, for `\end`
    /// mismatch diagnostics.
    env_stack: Vec<Token>,
    /// `\newcounter` bookkeeping: name -> parent counter name (for
    /// `[within]`); reset-on-parent-step propagation is not implemented
    /// yet (see CONTRACT.md), this is informational only.
    counter_parents: HashMap<String, Option<String>>,
    next_free_register: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Vertical,
    Horizontal,
    Math,
    InnerVertical,
    InnerHorizontal,
    InnerMath,
}

impl Mode {
    fn is_v(&self) -> bool {
        matches!(self, Mode::Vertical | Mode::InnerVertical)
    }
    fn is_h(&self) -> bool {
        matches!(self, Mode::Horizontal | Mode::InnerHorizontal)
    }
    fn is_m(&self) -> bool {
        matches!(self, Mode::Math | Mode::InnerMath)
    }
    fn is_inner(&self) -> bool {
        matches!(self, Mode::InnerVertical | Mode::InnerHorizontal | Mode::InnerMath)
    }
}

const PRIMITIVE_TABLE: &[(&str, Primitive)] = &[
    ("relax", Primitive::Relax),
    ("par", Primitive::Par),
    ("def", Primitive::Def),
    ("edef", Primitive::Edef),
    ("gdef", Primitive::Gdef),
    ("xdef", Primitive::Xdef),
    ("let", Primitive::Let),
    ("futurelet", Primitive::Futurelet),
    ("global", Primitive::Global),
    ("long", Primitive::Long),
    ("outer", Primitive::Outer),
    ("expandafter", Primitive::Expandafter),
    ("noexpand", Primitive::Noexpand),
    ("csname", Primitive::Csname),
    ("endcsname", Primitive::Endcsname),
    ("string", Primitive::String),
    ("number", Primitive::Number),
    ("romannumeral", Primitive::Romannumeral),
    ("the", Primitive::The),
    ("begingroup", Primitive::Begingroup),
    ("endgroup", Primitive::Endgroup),
    ("aftergroup", Primitive::Aftergroup),
    ("catcode", Primitive::Catcode),
    ("makeatletter", Primitive::Makeatletter),
    ("makeatother", Primitive::Makeatother),
    ("if", Primitive::If),
    ("ifcat", Primitive::Ifcat),
    ("ifx", Primitive::Ifx),
    ("ifnum", Primitive::Ifnum),
    ("ifdim", Primitive::Ifdim),
    ("ifodd", Primitive::Ifodd),
    ("ifvmode", Primitive::Ifvmode),
    ("ifhmode", Primitive::Ifhmode),
    ("ifmmode", Primitive::Ifmmode),
    ("ifinner", Primitive::Ifinner),
    ("ifcase", Primitive::Ifcase),
    ("iftrue", Primitive::Iftrue),
    ("iffalse", Primitive::Iffalse),
    ("or", Primitive::Or),
    ("else", Primitive::Else),
    ("fi", Primitive::Fi),
    ("newif", Primitive::Newif),
    ("unless", Primitive::Unless),
    ("count", Primitive::Count),
    ("dimen", Primitive::Dimen),
    ("skip", Primitive::Skip),
    ("toks", Primitive::Toks),
    ("countdef", Primitive::Countdef),
    ("dimendef", Primitive::Dimendef),
    ("skipdef", Primitive::Skipdef),
    ("toksdef", Primitive::Toksdef),
    ("advance", Primitive::Advance),
    ("multiply", Primitive::Multiply),
    ("divide", Primitive::Divide),
    ("numexpr", Primitive::Numexpr),
    ("dimexpr", Primitive::Dimexpr),
    ("newcommand", Primitive::NewCommand),
    ("renewcommand", Primitive::RenewCommand),
    ("providecommand", Primitive::ProvideCommand),
    ("newenvironment", Primitive::NewEnvironment),
    ("renewenvironment", Primitive::RenewEnvironment),
    ("begin", Primitive::Begin),
    ("end", Primitive::End),
    ("newcounter", Primitive::NewCounter),
    ("setcounter", Primitive::SetCounter),
    ("addtocounter", Primitive::AddToCounter),
    ("stepcounter", Primitive::StepCounter),
    ("refstepcounter", Primitive::RefStepCounter),
    ("value", Primitive::Value),
    ("arabic", Primitive::Arabic),
    ("roman", Primitive::RomanLower),
    ("Roman", Primitive::RomanUpper),
    ("alph", Primitive::AlphLower),
    ("Alph", Primitive::AlphUpper),
    ("fnsymbol", Primitive::Fnsymbol),
    ("@ifnextchar", Primitive::IfNextChar),
    ("@ifstar", Primitive::IfStar),
    ("@namedef", Primitive::NameDef),
    ("@nameuse", Primitive::NameUse),
];

/// What one dispatch step produced.
enum Step {
    Emit(Token),
    Continue,
    Eof,
}

impl<'a> Engine<'a> {
    pub fn new(source: &'a str) -> Self {
        Self::with_limits(source, Limits::default())
    }

    pub fn with_limits(source: &'a str, limits: Limits) -> Self {
        let mut scopes = Scopes::new();
        for (name, prim) in PRIMITIVE_TABLE {
            scopes.assign_cs(name, Meaning::Primitive(*prim), true);
        }
        Engine {
            sources: vec![Input::Text(Lexer::new(source, 0))],
            next_source_id: 1,
            scopes,
            conditionals: ConditionalStack::default(),
            limits,
            steps: 0,
            diagnostics: Vec::new(),
            pending_global: false,
            pending_long: false,
            pending_outer: false,
            metrics: Box::new(DefaultFontMetrics),
            mode: Mode::Vertical,
            env_stack: Vec::new(),
            counter_parents: HashMap::new(),
            next_free_register: 1,
        }
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn set_font_metrics(&mut self, metrics: Box<dyn FontMetrics>) {
        self.metrics = metrics;
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    fn err(&mut self, msg: impl Into<String>, span: Span) {
        self.diagnostics.push(Diagnostic::error(msg, span));
    }

    // ---- raw token stream -------------------------------------------------

    fn push_tokens(&mut self, toks: Vec<Token>) {
        if toks.is_empty() {
            return;
        }
        let pend = toks.into_iter().map(|tok| Pending { tok, frozen: false }).collect();
        self.sources.push(Input::Toks(pend, 0));
    }

    fn push_frozen(&mut self, tok: Token) {
        self.sources.push(Input::Toks(vec![Pending { tok, frozen: true }], 0));
    }

    fn next_raw(&mut self) -> Option<Pending> {
        loop {
            match self.sources.last_mut()? {
                Input::Text(lexer) => {
                    if let Some(tok) = lexer.next_token(self.scopes.cat_table()) {
                        return Some(Pending { tok, frozen: false });
                    } else {
                        self.sources.pop();
                        continue;
                    }
                }
                Input::Toks(toks, pos) => {
                    if *pos < toks.len() {
                        let p = &toks[*pos];
                        *pos += 1;
                        return Some(Pending { tok: p.tok.clone(), frozen: p.frozen });
                    } else {
                        self.sources.pop();
                        continue;
                    }
                }
            }
        }
    }

    fn next_raw_token(&mut self) -> Option<Token> {
        self.next_raw().map(|p| p.tok)
    }

    // ---- main dispatch loop -------------------------------------------

    /// Read and fully process the next content token: expands macros and
    /// expandable primitives, executes assignments/definitions/grouping,
    /// and returns the next token meant for the typesetting layer (or
    /// `None` at end of input).
    pub fn next_content_token(&mut self) -> Option<Token> {
        loop {
            self.steps += 1;
            if self.steps > self.limits.max_expansion_steps {
                self.err("expansion step limit exceeded (possible infinite macro loop)", Span::synthetic());
                return None;
            }
            let pending = self.next_raw()?;
            if pending.frozen {
                return Some(pending.tok);
            }
            match self.step(pending.tok) {
                Step::Emit(t) => return Some(t),
                Step::Continue => continue,
                Step::Eof => return None,
            }
        }
    }

    /// Drain the whole input into a token vector plus diagnostics. This is
    /// the primary library entry point; see `lib.rs::expand_str`.
    pub fn run(&mut self) -> Vec<Token> {
        let mut out = Vec::new();
        while let Some(tok) = self.next_content_token() {
            out.push(tok);
            if out.len() as u64 > self.limits.max_output_tokens {
                self.err("output token limit exceeded", Span::synthetic());
                break;
            }
        }
        out
    }

    fn step(&mut self, tok: Token) -> Step {
        match tok.kind.clone() {
            TokenKind::Char(_, _) => {
                if let Some(step) = self.maybe_handle_brace(&tok) {
                    step
                } else {
                    Step::Emit(tok)
                }
            }
            TokenKind::Param(_) => Step::Emit(tok),
            TokenKind::Eof => Step::Eof,
            TokenKind::ControlSequence(name) => {
                let meaning = self.scopes.meaning(&name);
                self.dispatch(tok, meaning)
            }
            TokenKind::ActiveChar(c) => {
                let meaning = self.scopes.active_meaning(c);
                match meaning {
                    Meaning::Undefined => Step::Emit(tok),
                    m => self.dispatch(tok, m),
                }
            }
        }
    }

    fn dispatch(&mut self, tok: Token, meaning: Meaning) -> Step {
        match meaning {
            Meaning::Macro(def) => {
                self.call_macro(&tok, &def);
                Step::Continue
            }
            Meaning::MacroWithOptional { body, default } => {
                self.call_macro_with_optional(&tok, &body, &default);
                Step::Continue
            }
            Meaning::Let(inner) => self.dispatch(tok, *inner),
            Meaning::CharLike(t) => Step::Emit(Token::new(t.kind, tok.span)),
            Meaning::RegisterAlias(kind, idx) => self.handle_register_ref(tok, kind, idx),
            Meaning::Primitive(p) => self.handle_primitive(tok, p),
            Meaning::Undefined => Step::Emit(tok),
        }
    }

    // ---- grouping (`{`/`}` characters) --------------------------------

    /// Characters with catcode BeginGroup/EndGroup open/close scopes just
    /// like `\begingroup`/`\endgroup`, but are still emitted as content
    /// tokens (typesetting needs to see the braces for e.g. `{\bf x}`).
    fn maybe_handle_brace(&mut self, tok: &Token) -> Option<Step> {
        if let TokenKind::Char(_, cat) = tok.kind {
            match cat {
                CatCode::BeginGroup => {
                    self.scopes.push_group();
                    return Some(Step::Emit(tok.clone()));
                }
                CatCode::EndGroup => {
                    let after = self.scopes.pop_group();
                    self.push_tokens(after);
                    return Some(Step::Emit(tok.clone()));
                }
                _ => {}
            }
        }
        None
    }

    // ---- macro definition & calling -----------------------------------

    /// Scan a `\def`-style parameter text up to (not including) the
    /// opening `{` of the body, per TeXbook p.203-205.
    fn scan_param_text(&mut self) -> (Vec<ParamPart>, MacroFlags) {
        let mut params = Vec::new();
        let mut flags = MacroFlags::default();
        loop {
            let tok = match self.next_raw_token() {
                Some(t) => t,
                None => break,
            };
            match &tok.kind {
                TokenKind::Char(_, CatCode::BeginGroup) => {
                    // pushed back: caller's scan_braced_body will re-read it
                    self.push_tokens(vec![tok]);
                    break;
                }
                TokenKind::Char(_, CatCode::Param) => {
                    // `#` in a parameter text is followed by a digit
                    // 1..9 naming the parameter slot (TeXbook p.203).
                    match self.next_raw_token() {
                        Some(next) => match next.kind {
                            TokenKind::Char(d, _) if d.is_ascii_digit() && d != '0' => {
                                let n = d.to_digit(10).unwrap() as u8;
                                // Look ahead: `#{` means brace-delimited
                                // last parameter (the `{` is left in the
                                // stream for scan_braced_group to consume
                                // as the start of the body).
                                if let Some(after) = self.next_raw_token() {
                                    if matches!(after.kind, TokenKind::Char(_, CatCode::BeginGroup)) {
                                        flags.brace_delimited_last = true;
                                    }
                                    self.push_tokens(vec![after]);
                                }
                                params.push(ParamPart::Param(n));
                            }
                            _ => {
                                self.err("parameter text: '#' must be followed by a digit 1-9", tok.span);
                                self.push_tokens(vec![next]);
                            }
                        },
                        None => {}
                    }
                }
                _ => params.push(ParamPart::Literal(tok)),
            }
        }
        (params, flags)
    }

    /// Scan a brace-delimited token list `{ ... }` with correct nested
    /// brace counting (TeXbook p.212's `scan_toks`), returning the inner
    /// tokens (braces not included). If `expand` is set, tokens are read
    /// through full content-token dispatch (for `\edef`/`\xdef`), honoring
    /// `\noexpand` freezing; otherwise tokens are taken completely raw
    /// (for `\def`/`\gdef` bodies, and for ordinary `{...}` arguments).
    fn scan_braced_group(&mut self, expand: bool) -> Vec<Token> {
        // Expect and consume the opening brace (skip intervening spaces).
        loop {
            match self.next_raw_token() {
                Some(t) => match t.kind {
                    TokenKind::Char(_, CatCode::Space) => continue,
                    TokenKind::Char(_, CatCode::BeginGroup) => break,
                    _ => {
                        // Missing '{': treat the single token as the whole
                        // group rather than panicking.
                        return vec![t];
                    }
                },
                None => return Vec::new(),
            }
        }
        let mut depth = 1i32;
        let mut out = Vec::new();
        loop {
            let next = if expand { self.next_expanding_raw() } else { self.next_raw() };
            let pending = match next {
                Some(p) => p,
                None => {
                    self.err("runaway argument / file ended inside a group (missing '}')", Span::synthetic());
                    break;
                }
            };
            match &pending.tok.kind {
                TokenKind::Char(_, CatCode::BeginGroup) => {
                    depth += 1;
                    out.push(pending.tok);
                }
                TokenKind::Char(_, CatCode::EndGroup) => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    out.push(pending.tok);
                }
                _ => out.push(pending.tok),
            }
        }
        out
    }

    /// Like `next_raw` but expands expandable tokens one at a time
    /// (respecting `\noexpand` freezing), used while scanning `\edef`
    /// bodies and other "expanded" argument contexts. Non-expandable
    /// tokens (including grouping braces, which the caller inspects) are
    /// returned as-is without executing assignments -- so this does NOT
    /// invoke `\def`/`\let`/etc as side effects; it only expands macros
    /// and expandable primitives. Assignment primitives found here are
    /// passed through literally, matching real `\edef`'s behavior of not
    /// executing `\def` embedded in its own body.
    fn next_expanding_raw(&mut self) -> Option<Pending> {
        loop {
            let pending = self.next_raw()?;
            if pending.frozen {
                return Some(pending);
            }
            let expandable = match &pending.tok.kind {
                TokenKind::ControlSequence(name) => matches!(
                    self.scopes.meaning(name),
                    Meaning::Macro(_)
                        | Meaning::Primitive(
                            Primitive::Expandafter
                                | Primitive::Noexpand
                                | Primitive::Csname
                                | Primitive::String
                                | Primitive::Number
                                | Primitive::Romannumeral
                                | Primitive::The
                                | Primitive::If
                                | Primitive::Ifcat
                                | Primitive::Ifx
                                | Primitive::Ifnum
                                | Primitive::Ifdim
                                | Primitive::Ifodd
                                | Primitive::Ifvmode
                                | Primitive::Ifhmode
                                | Primitive::Ifmmode
                                | Primitive::Ifinner
                                | Primitive::Ifcase
                                | Primitive::Iftrue
                                | Primitive::Iffalse
                                | Primitive::Or
                                | Primitive::Else
                                | Primitive::Fi
                                | Primitive::Unless
                        )
                ),
                TokenKind::ActiveChar(c) => !matches!(self.scopes.active_meaning(*c), Meaning::Undefined),
                _ => false,
            };
            if !expandable {
                return Some(pending);
            }
            match self.step(pending.tok) {
                Step::Emit(t) => return Some(Pending { tok: t, frozen: false }),
                Step::Continue => continue,
                Step::Eof => return None,
            }
        }
    }

    fn do_def(&mut self, kind: Primitive) {
        let global = self.pending_global || matches!(kind, Primitive::Gdef | Primitive::Xdef);
        let expand_body = matches!(kind, Primitive::Edef | Primitive::Xdef);
        let flags0 = MacroFlags { long: self.pending_long, outer: self.pending_outer, brace_delimited_last: false };
        self.pending_long = false;
        self.pending_outer = false;
        self.pending_global = false;

        let name_tok = match self.next_raw_token() {
            Some(t) => t,
            None => return,
        };
        let (params, flags1) = self.scan_param_text();
        let flags = MacroFlags { long: flags0.long, outer: flags0.outer, brace_delimited_last: flags1.brace_delimited_last };
        let body_toks = fold_param_tokens(self.scan_braced_group(expand_body));
        let arity = params
            .iter()
            .filter_map(|p| if let ParamPart::Param(n) = p { Some(*n) } else { None })
            .max()
            .unwrap_or(0);
        let body: Vec<BodyPart> = body_toks
            .into_iter()
            .map(|t| match t.kind {
                TokenKind::Param(n) => BodyPart::Param(n),
                _ => BodyPart::Literal(t),
            })
            .collect();
        let def = Rc::new(MacroDef { params, body, flags, arity });
        self.define_cs_token(&name_tok, Meaning::Macro(def), global);
    }

    fn define_cs_token(&mut self, name_tok: &Token, meaning: Meaning, global: bool) {
        match &name_tok.kind {
            TokenKind::ControlSequence(name) => self.scopes.assign_cs(name, meaning, global),
            TokenKind::ActiveChar(c) => self.scopes.assign_active(*c, meaning, global),
            _ => self.err("expected a control sequence or active character to define", name_tok.span),
        }
    }

    fn call_macro(&mut self, call_tok: &Token, def: &Rc<MacroDef>) {
        let mut args: HashMap<u8, Vec<Token>> = HashMap::new();
        let mut i = 0usize;
        while i < def.params.len() {
            match &def.params[i] {
                ParamPart::Literal(lit) => {
                    // Delimiter: must match the next raw token exactly.
                    match self.next_raw_token() {
                        Some(t) if t.same_token(lit) => {}
                        Some(_) | None => {
                            self.err("use of macro does not match its definition (delimiter mismatch)", call_tok.span);
                        }
                    }
                    i += 1;
                }
                ParamPart::Param(n) => {
                    // Determine if this parameter is delimited by looking
                    // at the following param-text entries.
                    let mut delim: Vec<Token> = Vec::new();
                    let mut j = i + 1;
                    while let Some(ParamPart::Literal(lit)) = def.params.get(j) {
                        delim.push(lit.clone());
                        j += 1;
                    }
                    let brace_delim_last = def.flags.brace_delimited_last && j == def.params.len();
                    let arg = if delim.is_empty() && !brace_delim_last {
                        self.scan_undelimited_arg()
                    } else if brace_delim_last {
                        self.scan_braced_group(false)
                    } else {
                        self.scan_delimited_arg(&delim)
                    };
                    args.insert(*n, arg);
                    // `scan_delimited_arg` already consumed the following
                    // literal delimiter tokens from the input, so skip
                    // past those same `ParamPart::Literal` entries in the
                    // param text instead of matching them again.
                    i = if delim.is_empty() { i + 1 } else { j };
                }
            }
        }
        let expansion = substitute_body(&def.body, &args);
        self.push_tokens(expansion);
    }

    /// Call a `\newcommand`-with-optional-first-argument-style macro:
    /// `#1` is either the bracketed `[...]` following the call site, or
    /// `default` when no `[` is present (the call site's next token is
    /// left untouched in that case).
    fn call_macro_with_optional(&mut self, _call_tok: &Token, def: &Rc<MacroDef>, default: &[Token]) {
        let mut args: HashMap<u8, Vec<Token>> = HashMap::new();
        let first = if let Some(t) = self.peek_one() {
            if matches!(t.kind, TokenKind::Char('[', CatCode::Other)) {
                self.next_raw_token();
                self.scan_bracketed_optional()
            } else {
                default.to_vec()
            }
        } else {
            default.to_vec()
        };
        args.insert(1, first);
        for n in 2..=def.arity {
            let arg = self.scan_undelimited_arg();
            args.insert(n, arg);
        }
        let expansion = substitute_body(&def.body, &args);
        self.push_tokens(expansion);
    }

    /// Scan `[...]` (opening bracket already consumed), tracking nested
    /// `[`/`]` and `{`/`}` so braced content inside the optional argument
    /// is not mistaken for the closing bracket.
    fn scan_bracketed_optional(&mut self) -> Vec<Token> {
        let mut out = Vec::new();
        let mut bracket_depth = 1i32;
        let mut brace_depth = 0i32;
        loop {
            let t = match self.next_raw_token() {
                Some(t) => t,
                None => {
                    self.err("runaway optional argument (missing ']')", Span::synthetic());
                    break;
                }
            };
            match &t.kind {
                TokenKind::Char('[', CatCode::Other) if brace_depth == 0 => {
                    bracket_depth += 1;
                    out.push(t);
                }
                TokenKind::Char(']', CatCode::Other) if brace_depth == 0 => {
                    bracket_depth -= 1;
                    if bracket_depth == 0 {
                        break;
                    }
                    out.push(t);
                }
                TokenKind::Char(_, CatCode::BeginGroup) => {
                    brace_depth += 1;
                    out.push(t);
                }
                TokenKind::Char(_, CatCode::EndGroup) => {
                    brace_depth -= 1;
                    out.push(t);
                }
                _ => out.push(t),
            }
        }
        out
    }

    fn scan_undelimited_arg(&mut self) -> Vec<Token> {
        // Skip leading spaces (TeXbook: spaces are ignored before an
        // undelimited argument).
        loop {
            match self.next_raw_token() {
                Some(t) => match t.kind {
                    TokenKind::Char(_, CatCode::Space) => continue,
                    TokenKind::Char(_, CatCode::BeginGroup) => {
                        self.push_tokens(vec![t]);
                        return self.scan_braced_group(false);
                    }
                    _ => return vec![t],
                },
                None => return Vec::new(),
            }
        }
    }

    fn scan_delimited_arg(&mut self, delim: &[Token]) -> Vec<Token> {
        let mut out = Vec::new();
        let mut brace_depth = 0i32;
        loop {
            // Try to match the delimiter at brace depth 0.
            if brace_depth == 0 && self.peek_matches(delim) {
                self.consume_n(delim.len());
                break;
            }
            match self.next_raw_token() {
                Some(t) => {
                    match t.kind {
                        TokenKind::Char(_, CatCode::BeginGroup) => brace_depth += 1,
                        TokenKind::Char(_, CatCode::EndGroup) => brace_depth -= 1,
                        _ => {}
                    }
                    out.push(t);
                }
                None => {
                    self.err("runaway argument (delimiter never found)", Span::synthetic());
                    break;
                }
            }
        }
        // Strip one matching outer brace pair, per TeX's rule that a
        // delimited argument enclosed in its own braces has them removed.
        if out.len() >= 2 {
            if let (TokenKind::Char(_, CatCode::BeginGroup), TokenKind::Char(_, CatCode::EndGroup)) =
                (&out.first().unwrap().kind, &out.last().unwrap().kind)
            {
                out = out[1..out.len() - 1].to_vec();
            }
        }
        out
    }

    fn peek_matches(&mut self, delim: &[Token]) -> bool {
        // Peek by pulling raw tokens into a buffer, then pushing them back.
        let mut buf = Vec::with_capacity(delim.len());
        for expected in delim {
            match self.next_raw_token() {
                Some(t) => {
                    let ok = t.same_token(expected);
                    buf.push(t);
                    if !ok {
                        self.push_tokens(buf);
                        return false;
                    }
                }
                None => {
                    self.push_tokens(buf);
                    return false;
                }
            }
        }
        self.push_tokens(buf);
        true
    }

    fn consume_n(&mut self, n: usize) {
        for _ in 0..n {
            self.next_raw_token();
        }
    }

    // ---- \let / \futurelet ---------------------------------------------

    fn do_let(&mut self) {
        let global = self.pending_global;
        self.pending_global = false;
        let name_tok = match self.next_raw_token() {
            Some(t) => t,
            None => return,
        };
        // optional spaces, one optional '=', one optional space
        self.skip_spaces();
        if let Some(t) = self.peek_one() {
            if let TokenKind::Char('=', CatCode::Other) = t.kind {
                self.next_raw_token();
                if let Some(t2) = self.peek_one() {
                    if let TokenKind::Char(_, CatCode::Space) = t2.kind {
                        self.next_raw_token();
                    }
                }
            }
        }
        let rhs = match self.next_raw_token() {
            Some(t) => t,
            None => return,
        };
        let meaning = self.meaning_of_token(&rhs);
        self.define_cs_token(&name_tok, meaning, global);
    }

    fn do_futurelet(&mut self) {
        let global = self.pending_global;
        self.pending_global = false;
        let name_tok = match self.next_raw_token() {
            Some(t) => t,
            None => return,
        };
        let t1 = self.next_raw_token();
        let t2 = self.next_raw_token();
        if let Some(t2) = &t2 {
            let meaning = self.meaning_of_token(t2);
            self.define_cs_token(&name_tok, meaning, global);
        }
        let mut reinsert = Vec::new();
        if let Some(t1) = t1 {
            reinsert.push(t1);
        }
        if let Some(t2) = t2 {
            reinsert.push(t2);
        }
        self.push_tokens(reinsert);
    }

    fn meaning_of_token(&self, tok: &Token) -> Meaning {
        match &tok.kind {
            TokenKind::ControlSequence(name) => self.scopes.meaning(name),
            TokenKind::ActiveChar(c) => self.scopes.active_meaning(*c),
            _ => Meaning::CharLike(tok.clone()),
        }
    }

    fn skip_spaces(&mut self) {
        loop {
            match self.peek_one() {
                Some(t) if matches!(t.kind, TokenKind::Char(_, CatCode::Space)) => {
                    self.next_raw_token();
                }
                _ => break,
            }
        }
    }

    fn peek_one(&mut self) -> Option<Token> {
        let t = self.next_raw_token()?;
        self.push_tokens(vec![t.clone()]);
        Some(t)
    }

    // ---- primitive handling --------------------------------------------

    fn handle_primitive(&mut self, tok: Token, p: Primitive) -> Step {
        use Primitive::*;
        match p {
            Relax => Step::Emit(tok),
            Par => Step::Emit(tok),
            Def | Edef | Gdef | Xdef => {
                self.do_def(p);
                Step::Continue
            }
            Let => {
                self.do_let();
                Step::Continue
            }
            Futurelet => {
                self.do_futurelet();
                Step::Continue
            }
            Global => {
                self.pending_global = true;
                Step::Continue
            }
            Long => {
                self.pending_long = true;
                Step::Continue
            }
            Outer => {
                self.pending_outer = true;
                Step::Continue
            }
            Expandafter => {
                let t1 = self.next_raw_token();
                let t2 = self.next_raw();
                if let Some(p2) = t2 {
                    if !p2.frozen {
                        match self.step(p2.tok) {
                            Step::Emit(t) => self.push_tokens(vec![t]),
                            Step::Continue => {}
                            Step::Eof => {}
                        }
                    } else {
                        self.push_frozen(p2.tok);
                    }
                }
                if let Some(t1) = t1 {
                    self.push_tokens(vec![t1]);
                }
                Step::Continue
            }
            Noexpand => {
                if let Some(t) = self.next_raw_token() {
                    self.push_frozen(t);
                }
                Step::Continue
            }
            Csname => {
                let mut name = ::std::string::String::new();
                loop {
                    match self.next_expanding_raw() {
                        Some(p) if p.tok.is_cs("endcsname") => break,
                        Some(p) => match p.tok.kind {
                            TokenKind::Char(c, _) => name.push(c),
                            _ => {
                                self.err("invalid token inside \\csname...\\endcsname", p.tok.span);
                            }
                        },
                        None => {
                            self.err("missing \\endcsname inserted (runaway \\csname)", tok.span);
                            break;
                        }
                    }
                }
                if matches!(self.scopes.meaning(&name), Meaning::Undefined) {
                    self.scopes.assign_cs(&name, Meaning::Primitive(Primitive::Relax), false);
                }
                self.push_tokens(vec![Token::new(TokenKind::ControlSequence(name), tok.span)]);
                Step::Continue
            }
            Endcsname => {
                self.err("extra \\endcsname", tok.span);
                Step::Continue
            }
            String => {
                if let Some(t) = self.next_raw_token() {
                    let s = self.string_of(&t);
                    self.push_tokens(chars_as_other(&s, tok.span));
                }
                Step::Continue
            }
            Number => {
                let n = self.scan_number();
                self.push_tokens(chars_as_other(&n.to_string(), tok.span));
                Step::Continue
            }
            Romannumeral => {
                let n = self.scan_number();
                self.push_tokens(chars_as_other(&to_roman(n), tok.span));
                Step::Continue
            }
            The => {
                let toks = self.do_the();
                self.push_tokens(toks);
                Step::Continue
            }
            Begingroup => {
                self.scopes.push_group();
                Step::Continue
            }
            Endgroup => {
                let after = self.scopes.pop_group();
                self.push_tokens(after);
                Step::Continue
            }
            Aftergroup => {
                if let Some(t) = self.next_raw_token() {
                    self.scopes.queue_aftergroup(t);
                }
                Step::Continue
            }
            Catcode => {
                let global = self.pending_global;
                self.pending_global = false;
                let code = self.scan_number();
                self.expect_equals();
                let val = self.scan_number();
                if let (Some(ch), Some(cat)) = (char::from_u32(code as u32), CatCode::from_u8(val as u8)) {
                    self.scopes.set_catcode(ch, cat, global);
                } else {
                    self.err("invalid \\catcode assignment", tok.span);
                }
                Step::Continue
            }
            Makeatletter => {
                self.scopes.set_catcode('@', CatCode::Letter, self.pending_global);
                self.pending_global = false;
                Step::Continue
            }
            Makeatother => {
                self.scopes.set_catcode('@', CatCode::Other, self.pending_global);
                self.pending_global = false;
                Step::Continue
            }
            If | Ifcat | Ifx | Ifnum | Ifdim | Ifodd | Ifvmode | Ifhmode | Ifmmode | Ifinner | Ifcase | Iftrue
            | Iffalse => {
                self.do_conditional(p, false);
                Step::Continue
            }
            Unless => {
                // `\unless\ifnum...`: only defined over the boolean-valued
                // (non-\ifcase) conditionals per e-TeX.
                if let Some(next) = self.next_raw_token() {
                    if let TokenKind::ControlSequence(name) = &next.kind {
                        if let Meaning::Primitive(inner) = self.scopes.meaning(name) {
                            self.do_conditional(inner, true);
                            return Step::Continue;
                        }
                    }
                    self.err("\\unless must be followed by a boolean \\if primitive", next.span);
                }
                Step::Continue
            }
            Or | Else | Fi => {
                self.handle_stray_or_else_fi(p, tok);
                Step::Continue
            }
            Newif => {
                self.do_newif();
                Step::Continue
            }
            Count | Dimen | Skip | Toks => {
                let idx = self.scan_number() as u16;
                self.finish_register_assignment_or_pass(
                    tok,
                    match p {
                        Count => RegisterKind::Count,
                        Dimen => RegisterKind::Dimen,
                        Skip => RegisterKind::Skip,
                        _ => RegisterKind::Toks,
                    },
                    idx,
                )
            }
            Countdef | Dimendef | Skipdef | Toksdef => {
                let global = self.pending_global;
                self.pending_global = false;
                let name_tok = self.next_raw_token();
                self.expect_equals();
                let idx = self.scan_number() as u16;
                let kind = match p {
                    Countdef => RegisterKind::Count,
                    Dimendef => RegisterKind::Dimen,
                    Skipdef => RegisterKind::Skip,
                    _ => RegisterKind::Toks,
                };
                if let Some(nt) = name_tok {
                    self.define_cs_token(&nt, Meaning::RegisterAlias(kind, idx), global);
                }
                Step::Continue
            }
            Advance | Multiply | Divide => {
                self.do_arith(p);
                Step::Continue
            }
            Numexpr | Dimexpr => {
                let v = self.scan_expr(matches!(p, Dimexpr));
                let s = if matches!(p, Dimexpr) { format!("{v}sp") } else { v.to_string() };
                // `\numexpr`/`\dimexpr` are read as internal quantities and
                // expand to their numeric text only via `\the`; used bare
                // they are still a value producer for register contexts.
                // For simplicity in this expansion-only engine we splice
                // the decimal text in as characters when encountered
                // directly (mirrors `\the\numexpr...\relax`).
                self.push_tokens(chars_as_other(&s, tok.span));
                Step::Continue
            }
            NewCommand | RenewCommand | ProvideCommand => {
                self.do_newcommand(p, tok.span);
                Step::Continue
            }
            NewEnvironment | RenewEnvironment => {
                self.do_newenvironment(tok.span);
                Step::Continue
            }
            Begin => {
                let name = self.read_name_arg();
                self.env_stack.push(Token::new(TokenKind::Char('e', CatCode::Other), tok.span));
                self.push_tokens(vec![Token::new(TokenKind::ControlSequence(name), tok.span)]);
                Step::Continue
            }
            End => {
                let name = self.read_name_arg();
                if self.env_stack.pop().is_none() {
                    self.err(format!("\\end{{{name}}} without matching \\begin"), tok.span);
                }
                self.push_tokens(vec![Token::new(TokenKind::ControlSequence(format!("end{name}")), tok.span)]);
                Step::Continue
            }
            NewCounter => {
                self.do_newcounter(tok.span);
                Step::Continue
            }
            SetCounter => {
                let name = self.read_name_arg();
                let v = self.scan_counter_value_arg();
                if let Some(idx) = self.counter_register(&name) {
                    self.scopes.set_count(idx, v, false);
                } else {
                    self.err(format!("\\setcounter on undefined counter '{name}'"), tok.span);
                }
                Step::Continue
            }
            AddToCounter => {
                let name = self.read_name_arg();
                let v = self.scan_counter_value_arg();
                if let Some(idx) = self.counter_register(&name) {
                    self.scopes.set_count(idx, self.scopes.count(idx) + v, false);
                } else {
                    self.err(format!("\\addtocounter on undefined counter '{name}'"), tok.span);
                }
                Step::Continue
            }
            StepCounter | RefStepCounter => {
                let name = self.read_name_arg();
                if let Some(idx) = self.counter_register(&name) {
                    self.scopes.set_count(idx, self.scopes.count(idx) + 1, false);
                } else {
                    self.err(format!("\\stepcounter on undefined counter '{name}'"), tok.span);
                }
                Step::Continue
            }
            Value => {
                let name = self.read_name_arg();
                self.push_tokens(vec![Token::new(TokenKind::ControlSequence(format!("c@{name}")), tok.span)]);
                Step::Continue
            }
            Arabic | RomanLower | RomanUpper | AlphLower | AlphUpper | Fnsymbol => {
                let name = self.read_name_arg();
                let v = self.counter_register(&name).map(|idx| self.scopes.count(idx)).unwrap_or(0);
                let s = match p {
                    Arabic => v.to_string(),
                    RomanLower => to_roman(v),
                    RomanUpper => to_roman(v).to_ascii_uppercase(),
                    AlphLower => to_alph(v, false),
                    AlphUpper => to_alph(v, true),
                    Fnsymbol => to_fnsymbol(v),
                    _ => unreachable!(),
                };
                self.push_tokens(chars_as_other(&s, tok.span));
                Step::Continue
            }
            IfNextChar => {
                let want = self.next_raw_token();
                let true_branch = self.scan_braced_group(false);
                let false_branch = self.scan_braced_group(false);
                let next = self.peek_one();
                let matched = match (&want, &next) {
                    (Some(w), Some(n)) => w.same_token(n),
                    _ => false,
                };
                self.push_tokens(if matched { true_branch } else { false_branch });
                Step::Continue
            }
            IfStar => {
                let true_branch = self.scan_braced_group(false);
                let false_branch = self.scan_braced_group(false);
                let is_star = matches!(self.peek_one().map(|t| t.kind), Some(TokenKind::Char('*', CatCode::Other)));
                if is_star {
                    self.next_raw_token();
                    self.push_tokens(true_branch);
                } else {
                    self.push_tokens(false_branch);
                }
                Step::Continue
            }
            NameDef => {
                let name = self.read_name_arg();
                let body_toks = self.scan_braced_group(false);
                let body: Vec<BodyPart> = body_toks.into_iter().map(BodyPart::Literal).collect();
                let def = Rc::new(MacroDef { params: Vec::new(), body, flags: MacroFlags::default(), arity: 0 });
                self.scopes.assign_cs(&name, Meaning::Macro(def), self.pending_global);
                self.pending_global = false;
                Step::Continue
            }
            NameUse => {
                let name = self.read_name_arg();
                if matches!(self.scopes.meaning(&name), Meaning::Undefined) {
                    self.err(format!("\\@nameuse of undefined \\{name}"), tok.span);
                }
                self.push_tokens(vec![Token::new(TokenKind::ControlSequence(name), tok.span)]);
                Step::Continue
            }
        }
    }

    /// Read a `{name}` argument and flatten it to a plain string (each
    /// inner token contributes its display character; used for
    /// environment/counter names which are always plain letters).
    fn read_name_arg(&mut self) -> String {
        self.scan_braced_group(false).iter().map(|t| t.display_name().replace('\\', "")).collect()
    }

    /// Read a `{...}` argument meant to hold a `<number>`-shaped value
    /// (used by `\setcounter`/`\addtocounter`): expands it, then parses
    /// the resulting characters as a decimal integer.
    fn scan_counter_value_arg(&mut self) -> i64 {
        let toks = self.scan_braced_group(true);
        let s: String = toks
            .iter()
            .filter_map(|t| if let TokenKind::Char(c, _) = t.kind { Some(c) } else { None })
            .collect();
        s.trim().parse().unwrap_or(0)
    }

    fn counter_register(&self, name: &str) -> Option<u16> {
        match self.scopes.meaning(&format!("c@{name}")) {
            Meaning::RegisterAlias(RegisterKind::Count, idx) => Some(idx),
            _ => None,
        }
    }

    fn do_newcounter(&mut self, span: Span) {
        let name = self.read_name_arg();
        // optional [within]
        let within = if let Some(t) = self.peek_one() {
            if matches!(t.kind, TokenKind::Char('[', CatCode::Other)) {
                self.next_raw_token();
                Some(self.scan_bracketed_optional().iter().map(|t| t.display_name().replace('\\', "")).collect::<String>())
            } else {
                None
            }
        } else {
            None
        };
        let idx = self.next_free_register;
        self.next_free_register += 1;
        self.scopes.assign_cs(&format!("c@{name}"), Meaning::RegisterAlias(RegisterKind::Count, idx), true);
        self.scopes.set_count(idx, 0, true);
        self.counter_parents.insert(name, within);
        let _ = span;
    }

    fn do_newcommand(&mut self, kind: Primitive, span: Span) {
        // optional leading '*' (a "robust" command marker in real LaTeX;
        // expansion behavior is identical either way for us).
        if let Some(t) = self.peek_one() {
            if matches!(t.kind, TokenKind::Char('*', CatCode::Other)) {
                self.next_raw_token();
            }
        }
        let name_tok = match self.next_raw_token() {
            Some(t) => t,
            None => return,
        };
        // `\newcommand{\foo}...` or `\newcommand\foo...`
        let name_tok = if matches!(name_tok.kind, TokenKind::Char(_, CatCode::BeginGroup)) {
            self.push_tokens(vec![name_tok]);
            let inner = self.scan_braced_group(false);
            inner.into_iter().next().unwrap_or(Token::synthetic(TokenKind::ControlSequence(String::new())))
        } else {
            name_tok
        };
        let already_defined = !matches!(self.meaning_of_token(&name_tok), Meaning::Undefined);
        match kind {
            Primitive::NewCommand if already_defined => {
                self.err(format!("\\newcommand cannot redefine existing command {}", name_tok.display_name()), span);
            }
            Primitive::RenewCommand if !already_defined => {
                self.err(format!("\\renewcommand cannot redefine undefined command {}", name_tok.display_name()), span);
            }
            _ => {}
        }
        // For \providecommand when already defined, we still parse (and
        // discard) the rest of the syntax below to keep the input stream
        // in sync with what real TeX would have consumed.
        let nargs = self.scan_optional_bracket_number();
        let default = if let Some(t) = self.peek_one() {
            if matches!(t.kind, TokenKind::Char('[', CatCode::Other)) {
                self.next_raw_token();
                Some(self.scan_bracketed_optional())
            } else {
                None
            }
        } else {
            None
        };
        let body_toks = fold_param_tokens(self.scan_braced_group(false));
        let arity = nargs.unwrap_or(0) as u8;
        let params: Vec<ParamPart> = (1..=arity).map(ParamPart::Param).collect();
        let body: Vec<BodyPart> = body_toks
            .into_iter()
            .map(|t| match t.kind {
                TokenKind::Param(n) => BodyPart::Param(n),
                _ => BodyPart::Literal(t),
            })
            .collect();
        if matches!(kind, Primitive::ProvideCommand) && already_defined {
            return;
        }
        let meaning = if let Some(default_toks) = default {
            let def = Rc::new(MacroDef { params, body, flags: MacroFlags::default(), arity });
            Meaning::MacroWithOptional { body: def, default: default_toks }
        } else {
            Meaning::Macro(Rc::new(MacroDef { params, body, flags: MacroFlags::default(), arity }))
        };
        self.define_cs_token(&name_tok, meaning, false);
    }

    fn scan_optional_bracket_number(&mut self) -> Option<i64> {
        if let Some(t) = self.peek_one() {
            if matches!(t.kind, TokenKind::Char('[', CatCode::Other)) {
                self.next_raw_token();
                let n = self.scan_number();
                // consume the closing ']'
                if let Some(t2) = self.peek_one() {
                    if matches!(t2.kind, TokenKind::Char(']', CatCode::Other)) {
                        self.next_raw_token();
                    }
                }
                return Some(n);
            }
        }
        None
    }

    fn do_newenvironment(&mut self, span: Span) {
        let name = self.read_name_arg();
        let nargs = self.scan_optional_bracket_number();
        let default = if let Some(t) = self.peek_one() {
            if matches!(t.kind, TokenKind::Char('[', CatCode::Other)) {
                self.next_raw_token();
                Some(self.scan_bracketed_optional())
            } else {
                None
            }
        } else {
            None
        };
        let begin_toks = fold_param_tokens(self.scan_braced_group(false));
        let end_toks = fold_param_tokens(self.scan_braced_group(false));
        let arity = nargs.unwrap_or(0) as u8;
        let params: Vec<ParamPart> = (1..=arity).map(ParamPart::Param).collect();
        let begin_body: Vec<BodyPart> = begin_toks
            .into_iter()
            .map(|t| match t.kind {
                TokenKind::Param(n) => BodyPart::Param(n),
                _ => BodyPart::Literal(t),
            })
            .collect();
        let end_body: Vec<BodyPart> =
            end_toks.into_iter().map(|t| match t.kind {
                TokenKind::Param(n) => BodyPart::Param(n),
                _ => BodyPart::Literal(t),
            }).collect();
        let begin_meaning = if let Some(default_toks) = default {
            let def = Rc::new(MacroDef { params, body: begin_body, flags: MacroFlags::default(), arity });
            Meaning::MacroWithOptional { body: def, default: default_toks }
        } else {
            Meaning::Macro(Rc::new(MacroDef { params, body: begin_body, flags: MacroFlags::default(), arity }))
        };
        self.scopes.assign_cs(&name, begin_meaning, true);
        self.scopes.assign_cs(
            &format!("end{name}"),
            Meaning::Macro(Rc::new(MacroDef { params: Vec::new(), body: end_body, flags: MacroFlags::default(), arity: 0 })),
            true,
        );
        let _ = span;
    }

    fn handle_register_ref(&mut self, tok: Token, kind: RegisterKind, idx: u16) -> Step {
        self.finish_register_assignment_or_pass(tok, kind, idx)
    }

    /// After reading `\count<idx>` (or a `\countdef`-alias), either this is
    /// an assignment (`<idx>=<value>`) or a read in a context like
    /// `\the\countN` (handled by `do_the`, which calls this same scan
    /// path). Since assignment vs. read cannot be told apart until we see
    /// (or fail to see) an `=`, we peek for `=`.
    fn finish_register_assignment_or_pass(&mut self, tok: Token, kind: RegisterKind, idx: u16) -> Step {
        let global = self.pending_global;
        self.skip_spaces();
        if let Some(t) = self.peek_one() {
            if matches!(t.kind, TokenKind::Char('=', CatCode::Other)) {
                self.next_raw_token();
                self.pending_global = false;
                match kind {
                    RegisterKind::Count => {
                        let v = self.scan_number();
                        self.scopes.set_count(idx, v, global);
                    }
                    RegisterKind::Dimen => {
                        let v = self.scan_dimen();
                        self.scopes.set_dimen(idx, v, global);
                    }
                    RegisterKind::Skip => {
                        let v = self.scan_glue();
                        self.scopes.set_skip(idx, v, global);
                    }
                    RegisterKind::Toks => {
                        let v = self.scan_braced_group(false);
                        self.scopes.set_toks(idx, v, global);
                    }
                }
                return Step::Continue;
            }
        }
        // Not an assignment: this register reference appeared as a bare
        // value (e.g. right after `\the`, or inside an expression); we
        // signal that by re-emitting a synthetic marker the caller
        // (`do_the`/`scan_expr`) already handled before calling us in
        // that context. When reached from ordinary content flow (a lone
        // `\count5` with no following `=`), that's a TeX error in real
        // TeX ("missing number" trying to use it as a command); we just
        // emit nothing to avoid corrupting output.
        self.err("register used without assignment (bare \\count/\\dimen/\\skip/\\toks outside \\the)", tok.span);
        Step::Continue
    }

    fn expect_equals(&mut self) {
        self.skip_spaces();
        if let Some(t) = self.peek_one() {
            if matches!(t.kind, TokenKind::Char('=', CatCode::Other)) {
                self.next_raw_token();
            }
        }
    }

    // ---- \the -------------------------------------------------------------

    fn do_the(&mut self) -> Vec<Token> {
        let tok = match self.next_raw_token() {
            Some(t) => t,
            None => return Vec::new(),
        };
        let (kind, idx) = match self.register_ref_of(&tok) {
            Some(pair) => pair,
            None => {
                self.err("\\the must be followed by an internal quantity", tok.span);
                return Vec::new();
            }
        };
        match kind {
            RegisterKind::Count => chars_as_other(&self.scopes.count(idx).to_string(), tok.span),
            RegisterKind::Dimen => chars_as_other(&format!("{}sp", self.scopes.dimen(idx)), tok.span),
            RegisterKind::Skip => {
                let g = self.scopes.skip(idx);
                chars_as_other(&format!("{}sp plus {}sp minus {}sp", g.value, g.stretch, g.shrink), tok.span)
            }
            RegisterKind::Toks => self.scopes.toks(idx),
        }
    }

    fn register_ref_of(&mut self, tok: &Token) -> Option<(RegisterKind, u16)> {
        match &tok.kind {
            TokenKind::ControlSequence(name) => match self.scopes.meaning(name) {
                Meaning::RegisterAlias(kind, idx) => Some((kind, idx)),
                Meaning::Primitive(p) => {
                    let kind = match p {
                        Primitive::Count => RegisterKind::Count,
                        Primitive::Dimen => RegisterKind::Dimen,
                        Primitive::Skip => RegisterKind::Skip,
                        Primitive::Toks => RegisterKind::Toks,
                        _ => return None,
                    };
                    let idx = self.scan_number() as u16;
                    Some((kind, idx))
                }
                _ => None,
            },
            _ => None,
        }
    }

    // ---- \advance / \multiply / \divide --------------------------------

    fn do_arith(&mut self, op: Primitive) {
        let global = self.pending_global;
        self.pending_global = false;
        let tok = match self.next_raw_token() {
            Some(t) => t,
            None => return,
        };
        let (kind, idx) = match self.register_ref_of(&tok) {
            Some(p) => p,
            None => {
                self.err("\\advance/\\multiply/\\divide must be followed by a register", tok.span);
                return;
            }
        };
        // optional "by"
        self.skip_spaces();
        self.maybe_consume_keyword("by");
        match op {
            Primitive::Advance => match kind {
                RegisterKind::Count => {
                    let d = self.scan_number();
                    self.scopes.set_count(idx, self.scopes.count(idx) + d, global);
                }
                RegisterKind::Dimen => {
                    let d = self.scan_dimen();
                    self.scopes.set_dimen(idx, self.scopes.dimen(idx) + d, global);
                }
                RegisterKind::Skip => {
                    let d = self.scan_glue();
                    let mut g = self.scopes.skip(idx);
                    g.value += d.value;
                    self.scopes.set_skip(idx, g, global);
                }
                RegisterKind::Toks => {}
            },
            Primitive::Multiply => {
                let d = self.scan_number();
                match kind {
                    RegisterKind::Count => self.scopes.set_count(idx, self.scopes.count(idx) * d, global),
                    RegisterKind::Dimen => self.scopes.set_dimen(idx, self.scopes.dimen(idx) * d, global),
                    RegisterKind::Skip => {
                        let mut g = self.scopes.skip(idx);
                        g.value *= d;
                        self.scopes.set_skip(idx, g, global);
                    }
                    RegisterKind::Toks => {}
                }
            }
            Primitive::Divide => {
                let d = self.scan_number();
                if d != 0 {
                    match kind {
                        RegisterKind::Count => self.scopes.set_count(idx, self.scopes.count(idx) / d, global),
                        RegisterKind::Dimen => self.scopes.set_dimen(idx, self.scopes.dimen(idx) / d, global),
                        RegisterKind::Skip => {
                            let mut g = self.scopes.skip(idx);
                            g.value /= d;
                            self.scopes.set_skip(idx, g, global);
                        }
                        RegisterKind::Toks => {}
                    }
                } else {
                    self.err("divide by zero", Span::synthetic());
                }
            }
            _ => unreachable!(),
        }
    }

    fn maybe_consume_keyword(&mut self, kw: &str) -> bool {
        let mut consumed = Vec::new();
        for expect in kw.chars() {
            match self.next_raw_token() {
                Some(t) => {
                    let matches_char = matches!(t.kind, TokenKind::Char(c, _) if c.eq_ignore_ascii_case(&expect));
                    consumed.push(t);
                    if !matches_char {
                        self.push_tokens(consumed);
                        return false;
                    }
                }
                None => {
                    self.push_tokens(consumed);
                    return false;
                }
            }
        }
        self.skip_spaces();
        true
    }

    // ---- number/dimen/glue scanning -------------------------------------

    /// Scan a `<number>` (TeXbook ch. 24): optional sign, then either an
    /// integer constant (decimal/octal `'`/hex `"`/char `` ` ``) or an
    /// internal quantity (register, `\numexpr`, ...), followed by one
    /// optional space which is silently absorbed.
    pub fn scan_number(&mut self) -> i64 {
        self.skip_spaces();
        let mut neg = false;
        loop {
            match self.peek_one() {
                Some(t) => match &t.kind {
                    TokenKind::Char('+', _) => {
                        self.next_raw_token();
                        self.skip_spaces();
                    }
                    TokenKind::Char('-', _) => {
                        self.next_raw_token();
                        neg = !neg;
                        self.skip_spaces();
                    }
                    _ => break,
                },
                None => break,
            }
        }
        let value = match self.peek_one() {
            Some(t) => match &t.kind {
                TokenKind::Char(c, CatCode::Other) if c.is_ascii_digit() => self.scan_decimal_digits(),
                TokenKind::Char('\'', _) => {
                    self.next_raw_token();
                    self.scan_radix_digits(8)
                }
                TokenKind::Char('"', _) => {
                    self.next_raw_token();
                    self.scan_radix_digits(16)
                }
                TokenKind::Char('`', _) => {
                    self.next_raw_token();
                    let c = self.next_raw_token();
                    match c.map(|t| t.kind) {
                        Some(TokenKind::Char(ch, _)) => ch as i64,
                        Some(TokenKind::ControlSequence(name)) if name.chars().count() == 1 => {
                            name.chars().next().unwrap() as i64
                        }
                        _ => 0,
                    }
                }
                TokenKind::ControlSequence(name) => match self.scopes.meaning(name) {
                    Meaning::RegisterAlias(RegisterKind::Count, idx) => {
                        self.next_raw_token();
                        self.scopes.count(idx)
                    }
                    Meaning::Primitive(Primitive::Count) => {
                        self.next_raw_token();
                        let idx = self.scan_number() as u16;
                        self.scopes.count(idx)
                    }
                    Meaning::Primitive(Primitive::Numexpr) => {
                        self.next_raw_token();
                        self.scan_expr(false)
                    }
                    _ => {
                        self.next_raw_token();
                        0
                    }
                },
                _ => 0,
            },
            None => 0,
        };
        self.skip_one_optional_space();
        if neg {
            -value
        } else {
            value
        }
    }

    fn scan_decimal_digits(&mut self) -> i64 {
        let mut s = String::new();
        loop {
            match self.peek_one() {
                Some(t) => match t.kind {
                    TokenKind::Char(c, CatCode::Other) if c.is_ascii_digit() => {
                        s.push(c);
                        self.next_raw_token();
                    }
                    _ => break,
                },
                None => break,
            }
        }
        s.parse().unwrap_or(0)
    }

    fn scan_radix_digits(&mut self, radix: u32) -> i64 {
        let mut s = String::new();
        loop {
            match self.peek_one() {
                Some(t) => match t.kind {
                    TokenKind::Char(c, _) if c.is_digit(radix) => {
                        s.push(c);
                        self.next_raw_token();
                    }
                    _ => break,
                },
                None => break,
            }
        }
        i64::from_str_radix(&s, radix).unwrap_or(0)
    }

    fn skip_one_optional_space(&mut self) {
        if let Some(t) = self.peek_one() {
            if matches!(t.kind, TokenKind::Char(_, CatCode::Space)) {
                self.next_raw_token();
            }
        }
    }

    /// Scan a `<dimen>` value in scaled points: `<number>` (possibly with a
    /// decimal point) followed by a unit. Font-relative `em`/`ex` use the
    /// engine's `FontMetrics`.
    pub fn scan_dimen(&mut self) -> i64 {
        self.skip_spaces();
        let mut neg = false;
        loop {
            match self.peek_one() {
                Some(t) => match t.kind {
                    TokenKind::Char('+', _) => {
                        self.next_raw_token();
                    }
                    TokenKind::Char('-', _) => {
                        self.next_raw_token();
                        neg = !neg;
                    }
                    _ => break,
                },
                None => break,
            }
        }
        // internal dimen register shortcut
        if let Some(t) = self.peek_one() {
            if let TokenKind::ControlSequence(name) = &t.kind {
                match self.scopes.meaning(name) {
                    Meaning::RegisterAlias(RegisterKind::Dimen, idx) => {
                        self.next_raw_token();
                        let v = self.scopes.dimen(idx);
                        return if neg { -v } else { v };
                    }
                    Meaning::Primitive(Primitive::Dimen) => {
                        self.next_raw_token();
                        let idx = self.scan_number() as u16;
                        let v = self.scopes.dimen(idx);
                        return if neg { -v } else { v };
                    }
                    Meaning::Primitive(Primitive::Dimexpr) => {
                        self.next_raw_token();
                        let v = self.scan_expr(true);
                        return if neg { -v } else { v };
                    }
                    _ => {}
                }
            }
        }
        let mut int_part = String::new();
        loop {
            match self.peek_one() {
                Some(t) => match t.kind {
                    TokenKind::Char(c, CatCode::Other) if c.is_ascii_digit() => {
                        int_part.push(c);
                        self.next_raw_token();
                    }
                    _ => break,
                },
                None => break,
            }
        }
        let mut frac = String::new();
        if let Some(t) = self.peek_one() {
            if matches!(t.kind, TokenKind::Char('.', _) | TokenKind::Char(',', _)) {
                self.next_raw_token();
                loop {
                    match self.peek_one() {
                        Some(t) => match t.kind {
                            TokenKind::Char(c, CatCode::Other) if c.is_ascii_digit() => {
                                frac.push(c);
                                self.next_raw_token();
                            }
                            _ => break,
                        },
                        None => break,
                    }
                }
            }
        }
        self.skip_spaces();
        let unit = self.read_unit_name();
        let int_val: i64 = int_part.parse().unwrap_or(0);
        let sp = match unit.as_str() {
            "em" => {
                let per = self.metrics.quad_sp() as f64;
                scale_decimal(int_val, &frac, per)
            }
            "ex" => {
                let per = self.metrics.x_height_sp() as f64;
                scale_decimal(int_val, &frac, per)
            }
            other => {
                let per = absolute_unit_sp_per_unit(other).unwrap_or(65536.0);
                scale_decimal(int_val, &frac, per)
            }
        };
        self.skip_one_optional_space();
        if neg {
            -sp
        } else {
            sp
        }
    }

    fn read_unit_name(&mut self) -> String {
        let mut s = String::new();
        for _ in 0..2 {
            match self.peek_one() {
                Some(t) => match t.kind {
                    TokenKind::Char(c, CatCode::Letter) => {
                        s.push(c);
                        self.next_raw_token();
                    }
                    _ => break,
                },
                None => break,
            }
        }
        s.to_ascii_lowercase()
    }

    pub fn scan_glue(&mut self) -> Glue {
        let value = self.scan_dimen();
        let mut g = Glue::fixed(value);
        self.skip_spaces();
        if self.maybe_consume_keyword("plus") {
            let (v, fil) = self.scan_stretch_shrink();
            g.stretch = v;
            g.stretch_fil = fil;
        }
        self.skip_spaces();
        if self.maybe_consume_keyword("minus") {
            let (v, fil) = self.scan_stretch_shrink();
            g.shrink = v;
            g.shrink_fil = fil;
        }
        g
    }

    fn scan_stretch_shrink(&mut self) -> (i64, u8) {
        // Reuse scan_dimen's digit/frac scanning but allow "fil"+"l"*.
        let neg = false;
        let mut int_part = String::new();
        loop {
            match self.peek_one() {
                Some(t) => match t.kind {
                    TokenKind::Char(c, CatCode::Other) if c.is_ascii_digit() => {
                        int_part.push(c);
                        self.next_raw_token();
                    }
                    _ => break,
                },
                None => break,
            }
        }
        let mut frac = String::new();
        if let Some(t) = self.peek_one() {
            if matches!(t.kind, TokenKind::Char('.', _)) {
                self.next_raw_token();
                loop {
                    match self.peek_one() {
                        Some(t) => match t.kind {
                            TokenKind::Char(c, CatCode::Other) if c.is_ascii_digit() => {
                                frac.push(c);
                                self.next_raw_token();
                            }
                            _ => break,
                        },
                        None => break,
                    }
                }
            }
        }
        self.skip_spaces();
        if self.maybe_consume_keyword("fil") {
            let mut fil = 1u8;
            while self.maybe_consume_keyword("l") {
                fil += 1;
            }
            let int_val: i64 = int_part.parse().unwrap_or(0);
            let v = scale_decimal(int_val, &frac, 65536.0);
            return (if neg { -v } else { v }, fil);
        }
        let unit = self.read_unit_name();
        let per = absolute_unit_sp_per_unit(&unit).unwrap_or(65536.0);
        let int_val: i64 = int_part.parse().unwrap_or(0);
        let v = scale_decimal(int_val, &frac, per);
        (if neg { -v } else { v }, 0)
    }

    /// A minimal `\numexpr`/`\dimexpr` (e-TeX) evaluator: `+ - * /` with
    /// standard precedence and parentheses, operands are `<number>`
    /// (or `<dimen>` for dimexpr), terminated implicitly (no `\relax`
    /// required, though one is accepted and consumed if present).
    fn scan_expr(&mut self, is_dimen: bool) -> i64 {
        let v = self.expr_sum(is_dimen);
        if let Some(t) = self.peek_one() {
            if t.is_cs("relax") {
                self.next_raw_token();
            }
        }
        v
    }

    fn expr_sum(&mut self, is_dimen: bool) -> i64 {
        let mut acc = self.expr_prod(is_dimen);
        loop {
            self.skip_spaces();
            match self.peek_one() {
                Some(t) if matches!(t.kind, TokenKind::Char('+', _)) => {
                    self.next_raw_token();
                    acc += self.expr_prod(is_dimen);
                }
                Some(t) if matches!(t.kind, TokenKind::Char('-', _)) => {
                    self.next_raw_token();
                    acc -= self.expr_prod(is_dimen);
                }
                _ => break,
            }
        }
        acc
    }

    fn expr_prod(&mut self, is_dimen: bool) -> i64 {
        let mut acc = self.expr_atom(is_dimen);
        loop {
            self.skip_spaces();
            match self.peek_one() {
                Some(t) if matches!(t.kind, TokenKind::Char('*', _)) => {
                    self.next_raw_token();
                    acc *= self.expr_atom(is_dimen);
                }
                Some(t) if matches!(t.kind, TokenKind::Char('/', _)) => {
                    self.next_raw_token();
                    let d = self.expr_atom(is_dimen);
                    acc = if d != 0 { acc / d } else { 0 };
                }
                _ => break,
            }
        }
        acc
    }

    fn expr_atom(&mut self, is_dimen: bool) -> i64 {
        self.skip_spaces();
        if let Some(t) = self.peek_one() {
            if matches!(t.kind, TokenKind::Char('(', _)) {
                self.next_raw_token();
                let v = self.expr_sum(is_dimen);
                self.skip_spaces();
                if let Some(t2) = self.peek_one() {
                    if matches!(t2.kind, TokenKind::Char(')', _)) {
                        self.next_raw_token();
                    }
                }
                return v;
            }
        }
        if is_dimen {
            self.scan_dimen()
        } else {
            self.scan_number()
        }
    }

    // ---- conditionals -----------------------------------------------------

    fn do_conditional(&mut self, prim: Primitive, unless: bool) {
        use Primitive::*;
        if matches!(prim, Ifcase) {
            let n = self.scan_number();
            self.conditionals.push(IfShape::Case, IfBranch::Taken);
            let mut remaining = n;
            loop {
                if remaining <= 0 {
                    break;
                }
                match self.skip_to_or_else_fi() {
                    BranchEnd::Or => {
                        remaining -= 1;
                    }
                    BranchEnd::Else | BranchEnd::Fi => {
                        self.conditionals.pop();
                        return;
                    }
                }
            }
            return;
        }
        let truth = match prim {
            Iftrue => true,
            Iffalse => false,
            If => {
                let a = self.scan_if_char_token();
                let b = self.scan_if_char_token();
                a == b
            }
            Ifcat => {
                let a = self.scan_if_cat_token();
                let b = self.scan_if_cat_token();
                a == b
            }
            Ifx => self.scan_ifx(),
            Ifnum => {
                let a = self.scan_number();
                let rel = self.scan_relation();
                let b = self.scan_number();
                apply_relation(a, b, rel)
            }
            Ifdim => {
                let a = self.scan_dimen();
                let rel = self.scan_relation();
                let b = self.scan_dimen();
                apply_relation(a, b, rel)
            }
            Ifodd => self.scan_number() % 2 != 0,
            Ifvmode => self.mode.is_v(),
            Ifhmode => self.mode.is_h(),
            Ifmmode => self.mode.is_m(),
            Ifinner => self.mode.is_inner(),
            _ => unreachable!(),
        };
        let truth = if unless { !truth } else { truth };
        if truth {
            self.conditionals.push(IfShape::TwoWay, IfBranch::Taken);
        } else {
            self.conditionals.push(IfShape::TwoWay, IfBranch::Skipping);
            match self.skip_to_or_else_fi() {
                BranchEnd::Else => {
                    if let Some(f) = self.conditionals.top_mut() {
                        f.branch = IfBranch::Taken;
                    }
                }
                BranchEnd::Fi | BranchEnd::Or => {
                    self.conditionals.pop();
                }
            }
        }
    }

    /// `\if`: compare the character codes of the next two tokens after
    /// full expansion (each read as if by `\noexpand`-aware expansion),
    /// treating a control sequence as if it had char code 256 (never
    /// equal to any real character) unless it is `\noexpand`-frozen, per
    /// TeXbook rule 4.
    fn scan_if_char_token(&mut self) -> Option<char> {
        let p = self.next_expanding_raw()?;
        match p.tok.kind {
            TokenKind::Char(c, _) => Some(c),
            TokenKind::ActiveChar(c) => Some(c),
            _ => None,
        }
    }

    fn scan_if_cat_token(&mut self) -> Option<CatCode> {
        let p = self.next_expanding_raw()?;
        match p.tok.kind {
            TokenKind::Char(_, cat) => Some(cat),
            TokenKind::ActiveChar(_) => Some(CatCode::Active),
            _ => None,
        }
    }

    /// `\ifx`: compare the *meanings* of the next two tokens without
    /// expanding either (TeXbook rule 5): two macros are equal iff same
    /// param text & body & long/outer flags; two primitives/let-chars
    /// equal iff same underlying primitive/char; undefined equals
    /// undefined.
    fn scan_ifx(&mut self) -> bool {
        let t1 = self.next_raw_token();
        let t2 = self.next_raw_token();
        let (t1, t2) = match (t1, t2) {
            (Some(a), Some(b)) => (a, b),
            _ => return false,
        };
        let m1 = self.meaning_of_token(&t1);
        let m2 = self.meaning_of_token(&t2);
        meanings_equal(&m1, &m2)
    }

    fn scan_relation(&mut self) -> Relation {
        self.skip_spaces();
        match self.next_raw_token().map(|t| t.kind) {
            Some(TokenKind::Char('<', _)) => Relation::Lt,
            Some(TokenKind::Char('=', _)) => Relation::Eq,
            Some(TokenKind::Char('>', _)) => Relation::Gt,
            _ => Relation::Eq,
        }
    }

    /// Skip tokens (respecting nested nesting of any `\if...` we
    /// encounter, which must be balanced by their own `\fi`) until we hit
    /// an `\else`/`\or`/`\fi` that belongs to *this* conditional level.
    fn skip_to_or_else_fi(&mut self) -> BranchEnd {
        let mut depth = 0i32;
        loop {
            let tok = match self.next_raw_token() {
                Some(t) => t,
                None => {
                    self.err("file ended while skipping a conditional (missing \\fi)", Span::synthetic());
                    return BranchEnd::Fi;
                }
            };
            if let TokenKind::ControlSequence(name) = &tok.kind {
                match self.scopes.meaning(name) {
                    Meaning::Primitive(p) if is_if_primitive(p) => depth += 1,
                    Meaning::Primitive(Primitive::Fi) => {
                        if depth == 0 {
                            return BranchEnd::Fi;
                        }
                        depth -= 1;
                    }
                    Meaning::Primitive(Primitive::Else) if depth == 0 => return BranchEnd::Else,
                    Meaning::Primitive(Primitive::Or) if depth == 0 => return BranchEnd::Or,
                    _ => {}
                }
            }
        }
    }

    fn handle_stray_or_else_fi(&mut self, p: Primitive, tok: Token) {
        // Reaching `\else`/`\or`/`\fi` directly (not via skip_to_or_else_fi)
        // means we were in the *taken* branch and must now skip to `\fi`.
        match self.conditionals.pop() {
            Some(frame) => match p {
                Primitive::Fi => {} // branch simply ends here
                Primitive::Else | Primitive::Or => {
                    if matches!(frame.branch, IfBranch::Taken) {
                        // We were executing the taken branch; an `\else`
                        // or `\or` here means skip the remaining branches
                        // to the matching `\fi`.
                        self.skip_balanced_to_fi();
                    }
                }
                _ => unreachable!("handle_stray_or_else_fi is only called with Fi/Else/Or"),
            },
            None => {
                self.err(format!("extra {}", tok.display_name()), tok.span);
            }
        }
    }

    fn skip_balanced_to_fi(&mut self) {
        let mut depth = 0i32;
        loop {
            let tok = match self.next_raw_token() {
                Some(t) => t,
                None => return,
            };
            if let TokenKind::ControlSequence(name) = &tok.kind {
                match self.scopes.meaning(name) {
                    Meaning::Primitive(p) if is_if_primitive(p) => depth += 1,
                    Meaning::Primitive(Primitive::Fi) => {
                        if depth == 0 {
                            return;
                        }
                        depth -= 1;
                    }
                    _ => {}
                }
            }
        }
    }

    // ---- \newif -------------------------------------------------------

    fn do_newif(&mut self) {
        let global = self.pending_global;
        self.pending_global = false;
        let name_tok = match self.next_raw_token() {
            Some(t) => t,
            None => return,
        };
        let base = match &name_tok.kind {
            TokenKind::ControlSequence(n) => n.strip_prefix("if").unwrap_or(n).to_string(),
            _ => {
                self.err("\\newif requires a control sequence starting with \\if", name_tok.span);
                return;
            }
        };
        let if_name = format!("if{base}");
        let true_name = format!("{base}true");
        let false_name = format!("{base}false");
        // \iffoo := \iffalse initially, a plain alias for \iffalse's
        // primitive behavior (TeXbook: \newif makes a *fresh* conditional,
        // but for our engine we can safely alias it to \iffalse/\iftrue
        // and re-point it via the true/false macros below).
        self.scopes.assign_cs(&if_name, Meaning::Primitive(Primitive::Iffalse), global);
        let true_body = vec![BodyPart::Literal(Token::synthetic(TokenKind::ControlSequence("global".into()))),
            BodyPart::Literal(Token::synthetic(TokenKind::ControlSequence("let".into()))),
            BodyPart::Literal(Token::synthetic(TokenKind::ControlSequence(if_name.clone()))),
            BodyPart::Literal(Token::synthetic(TokenKind::ControlSequence("iftrue".into())))];
        let false_body = vec![BodyPart::Literal(Token::synthetic(TokenKind::ControlSequence("global".into()))),
            BodyPart::Literal(Token::synthetic(TokenKind::ControlSequence("let".into()))),
            BodyPart::Literal(Token::synthetic(TokenKind::ControlSequence(if_name.clone()))),
            BodyPart::Literal(Token::synthetic(TokenKind::ControlSequence("iffalse".into())))];
        self.scopes.assign_cs(
            &true_name,
            Meaning::Macro(Rc::new(MacroDef { params: Vec::new(), body: true_body, flags: MacroFlags::default(), arity: 0 })),
            global,
        );
        self.scopes.assign_cs(
            &false_name,
            Meaning::Macro(Rc::new(MacroDef { params: Vec::new(), body: false_body, flags: MacroFlags::default(), arity: 0 })),
            global,
        );
    }

    // ---- \string / \meaning helpers ------------------------------------

    fn string_of(&self, tok: &Token) -> String {
        match &tok.kind {
            TokenKind::ControlSequence(name) => format!("\\{name}"),
            TokenKind::ActiveChar(c) => c.to_string(),
            TokenKind::Char(c, _) => c.to_string(),
            _ => String::new(),
        }
    }

    /// `\meaning`: not a full primitive in the PRIMITIVE_TABLE above (it is
    /// rarely needed outside diagnostics/oracle comparison), exposed as a
    /// helper the oracle tests call directly via `Engine::meaning_string`.
    pub fn meaning_string(&self, tok: &Token) -> String {
        let m = self.meaning_of_token(tok);
        meaning_to_string(&m)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Relation {
    Lt,
    Eq,
    Gt,
}

fn apply_relation(a: i64, b: i64, rel: Relation) -> bool {
    match rel {
        Relation::Lt => a < b,
        Relation::Eq => a == b,
        Relation::Gt => a > b,
    }
}

enum BranchEnd {
    Or,
    Else,
    Fi,
}

fn is_if_primitive(p: Primitive) -> bool {
    use Primitive::*;
    matches!(
        p,
        If | Ifcat | Ifx | Ifnum | Ifdim | Ifodd | Ifvmode | Ifhmode | Ifmmode | Ifinner | Ifcase | Iftrue | Iffalse
    )
}

fn meanings_equal(a: &Meaning, b: &Meaning) -> bool {
    match (a, b) {
        (Meaning::Undefined, Meaning::Undefined) => true,
        (Meaning::Primitive(p1), Meaning::Primitive(p2)) => p1 == p2,
        (Meaning::CharLike(t1), Meaning::CharLike(t2)) => t1.kind == t2.kind,
        (Meaning::RegisterAlias(k1, i1), Meaning::RegisterAlias(k2, i2)) => k1 == k2 && i1 == i2,
        (Meaning::Macro(m1), Meaning::Macro(m2)) => {
            params_equal(&m1.params, &m2.params) && body_equal(&m1.body, &m2.body) && m1.flags == m2.flags
        }
        (
            Meaning::MacroWithOptional { body: b1, default: d1 },
            Meaning::MacroWithOptional { body: b2, default: d2 },
        ) => {
            params_equal(&b1.params, &b2.params)
                && body_equal(&b1.body, &b2.body)
                && b1.flags == b2.flags
                && d1.len() == d2.len()
                && d1.iter().zip(d2.iter()).all(|(a, b)| a.kind == b.kind)
        }
        (Meaning::Let(l1), other) => meanings_equal(l1, other),
        (other, Meaning::Let(l2)) => meanings_equal(other, l2),
        _ => false,
    }
}

/// Meaning-comparison (`\ifx`) equality of macro param/body lists must
/// ignore source spans -- two macros defined identically at different
/// source locations are still "the same" per TeXbook rule 5.
fn params_equal(a: &[ParamPart], b: &[ParamPart]) -> bool {
    a.len() == b.len()
        && a.iter().zip(b.iter()).all(|(x, y)| match (x, y) {
            (ParamPart::Literal(t1), ParamPart::Literal(t2)) => t1.kind == t2.kind,
            (ParamPart::Param(n1), ParamPart::Param(n2)) => n1 == n2,
            _ => false,
        })
}

fn body_equal(a: &[BodyPart], b: &[BodyPart]) -> bool {
    a.len() == b.len()
        && a.iter().zip(b.iter()).all(|(x, y)| match (x, y) {
            (BodyPart::Literal(t1), BodyPart::Literal(t2)) => t1.kind == t2.kind,
            (BodyPart::Param(n1), BodyPart::Param(n2)) => n1 == n2,
            _ => false,
        })
}

fn meaning_to_string(m: &Meaning) -> String {
    match m {
        Meaning::Undefined => "undefined".to_string(),
        Meaning::Primitive(p) => format!("{p:?}").to_ascii_lowercase(),
        Meaning::CharLike(t) => match t.kind {
            TokenKind::Char(c, cat) => format!("the character {c} (cat {})", cat as u8 as i32),
            _ => "the character".to_string(),
        },
        Meaning::RegisterAlias(k, idx) => format!("{k:?} register {idx}").to_ascii_lowercase(),
        Meaning::Macro(def) => {
            let mut s = String::from("macro:");
            for p in &def.params {
                match p {
                    ParamPart::Literal(t) => s.push_str(&t.display_name()),
                    ParamPart::Param(n) => s.push_str(&format!("#{n}")),
                }
            }
            s.push_str("->");
            for p in &def.body {
                match p {
                    BodyPart::Literal(t) => s.push_str(&t.display_name()),
                    BodyPart::Param(n) => s.push_str(&format!("#{n}")),
                }
            }
            s
        }
        Meaning::MacroWithOptional { body, .. } => format!("macro (optional-arg):arity {}", body.arity),
        Meaning::Let(inner) => meaning_to_string(inner),
    }
}

/// Fold `#` (catcode 6) characters in a raw macro-body token list into
/// `TokenKind::Param` placeholders: `#` followed by a digit 1-9 becomes
/// that parameter slot, `##` becomes one literal `#` (TeXbook p.204).
fn fold_param_tokens(tokens: Vec<Token>) -> Vec<Token> {
    let mut out = Vec::with_capacity(tokens.len());
    let mut i = 0usize;
    while i < tokens.len() {
        let t = &tokens[i];
        if let TokenKind::Char(_, CatCode::Param) = t.kind {
            if let Some(next) = tokens.get(i + 1) {
                if let TokenKind::Char(d, _) = next.kind {
                    if let Some(n) = d.to_digit(10) {
                        if (1..=9).contains(&n) {
                            out.push(Token::new(TokenKind::Param(n as u8), t.span));
                            i += 2;
                            continue;
                        }
                    }
                }
                if matches!(next.kind, TokenKind::Char(_, CatCode::Param)) {
                    out.push(Token::new(TokenKind::Char('#', CatCode::Param), t.span));
                    i += 2;
                    continue;
                }
            }
        }
        out.push(t.clone());
        i += 1;
    }
    out
}

fn substitute_body(body: &[BodyPart], args: &HashMap<u8, Vec<Token>>) -> Vec<Token> {
    let mut expansion = Vec::with_capacity(body.len());
    for part in body {
        match part {
            BodyPart::Literal(t) => expansion.push(t.clone()),
            BodyPart::Param(n) => {
                if let Some(a) = args.get(n) {
                    expansion.extend(a.iter().cloned());
                }
            }
        }
    }
    expansion
}

fn chars_as_other(s: &str, span: Span) -> Vec<Token> {
    s.chars()
        .map(|c| {
            let cat = if c == ' ' { CatCode::Space } else { CatCode::Other };
            Token::new(TokenKind::Char(c, cat), span)
        })
        .collect()
}

/// `\alph`/`\Alph`: 1->a, 2->b, ..., 26->z (LaTeX errors past 26; we clamp
/// silently rather than panic).
fn to_alph(n: i64, upper: bool) -> String {
    if n < 1 || n > 26 {
        return String::new();
    }
    let base = if upper { b'A' } else { b'a' };
    ((base + (n as u8 - 1)) as char).to_string()
}

/// `\fnsymbol`: the 9 standard footnote symbols. Real LaTeX renders
/// special glyphs (asterisk, dagger, double-dagger, section, paragraph,
/// double-vertical-bar, then doubled versions); we use ASCII
/// approximations since we have no math/symbol font access here --
/// documented as an approximation in CONTRACT.md.
fn to_fnsymbol(n: i64) -> String {
    const SYMS: &[&str] = &["*", "**", "***", "+", "++", "+++", "#", "##", "###"];
    if n >= 1 && (n as usize) <= SYMS.len() {
        SYMS[n as usize - 1].to_string()
    } else {
        String::new()
    }
}

fn to_roman(mut n: i64) -> String {
    if n <= 0 {
        return String::new();
    }
    const VALUES: &[(i64, &str)] = &[
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
    let mut out = String::new();
    for (v, s) in VALUES {
        while n >= *v {
            out.push_str(s);
            n -= *v;
        }
    }
    out
}
