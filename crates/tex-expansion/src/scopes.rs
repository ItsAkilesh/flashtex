//! Unified grouping/save-stack for everything TeX assignments can touch
//! locally: control-sequence meanings (`\def`/`\let`/...), category codes
//! (`\catcode`), `\uccode`/`\lccode`, integer parameters (`\escapechar`),
//! and registers (`\count`/`\dimen`/`\skip`/`\toks`).
//! TeXbook ch. 24's "save stack": `{`/`}` and `\begingroup`/`\endgroup`
//! open/close a scope; assignments are local by default and are undone
//! when their scope closes; `\global` makes them permanent (no save
//! entry is pushed) and `\aftergroup` queues a token for reinsertion right
//! after the scope closes.
//!
//! Everything here is `Clone` (cheaply: macro bodies are behind `Rc`) so
//! the incremental expander can snapshot the whole assignment state at a
//! checkpoint.

use std::collections::HashMap;
use std::rc::Rc;

use crate::catcode::{CatCode, CatCodeTable};
use crate::macro_def::MacroDef;
use crate::registers::Glue;
use crate::span::Span;
use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Meaning {
    Macro(Rc<MacroDef>),
    Let(Box<Meaning>),
    CharLike(Token),
    Primitive(Primitive),
    RegisterAlias(RegisterKind, u16),
    /// `\chardef\x=<n>`: behaves as the character `<n>` when typeset and
    /// as the integer `<n>` in a `<number>` context (TeXbook p. 277).
    CharDef(i64),
    /// `\mathchardef\x=<n>`: a math character code; an integer in
    /// `<number>` contexts.
    MathCharDef(i64),
    Undefined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterKind {
    Count,
    Dimen,
    Skip,
    Toks,
}

/// Integer parameters this crate actually models (the rest of TeX's
/// `\tolerance`-style typesetting parameters are passed through to the
/// typesetter untouched).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntParam {
    Escapechar,
    Endlinechar,
    Newlinechar,
    /// e-TeX `\eTeXversion` (read-only in TeX; 2).
    ETeXVersion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Primitive {
    /// A host-typeset command (`Engine::declare_host_command`): defined, but
    /// emitted unchanged.
    Host,
    Relax,
    Par,
    Def,
    Edef,
    Gdef,
    Xdef,
    Let,
    Futurelet,
    Global,
    Long,
    Outer,
    Protected,
    Expandafter,
    Noexpand,
    Csname,
    Endcsname,
    String,
    Number,
    Romannumeral,
    MeaningOf,
    The,
    Unexpanded,
    Detokenize,
    /// pdfTeX/e-TeX 2019 `\expanded`.
    Expanded,
    // -- file/terminal I/O. `\input` reads through the host's file reader
    // when one is set (and otherwise passes through); the rest are
    // side-effect-free stand-ins so format code can be expanded.
    InputFile,
    Immediate,
    Write,
    Openout,
    Closeout,
    Openin,
    Closein,
    Read,
    Message,
    Errmessage,
    Jobname,
    /// e-TeX `\eTeXrevision` (expands to `.6`).
    ETeXRevision,
    /// pdfTeX `\pdfstrcmp` (XeTeX `\strcmp`).
    Pdfstrcmp,
    Scantokens,
    Afterassignment,
    Uppercase,
    Lowercase,
    Uccode,
    Lccode,
    Chardef,
    Mathchardef,
    IntPar(IntParam),
    Begingroup,
    Endgroup,
    Aftergroup,
    Catcode,
    Ignorespaces,
    Endinput,
    If,
    Ifcat,
    Ifx,
    Ifnum,
    Ifdim,
    Ifodd,
    Ifvmode,
    Ifhmode,
    Ifmmode,
    Ifinner,
    Ifcase,
    Iftrue,
    Iffalse,
    Ifdefined,
    Ifcsname,
    Ifhbox,
    Ifvbox,
    Ifvoid,
    Ifeof,
    Ifincsname,
    Or,
    Else,
    Fi,
    Newif,
    Unless,
    Count,
    Dimen,
    Skip,
    Toks,
    Countdef,
    Dimendef,
    Skipdef,
    Toksdef,
    Newcount,
    Newdimen,
    Newskip,
    Newtoks,
    Advance,
    Multiply,
    Divide,
    Numexpr,
    Dimexpr,
    // -- LaTeX layer (built on the primitives above) --
    NewCommand,
    RenewCommand,
    ProvideCommand,
    DeclareRobustCommand,
    NewEnvironment,
    RenewEnvironment,
    Begin,
    End,
    NewCounter,
    SetCounter,
    AddToCounter,
    StepCounter,
    RefStepCounter,
    AddToReset,
    RemoveFromReset,
    CounterWithin,
    CounterWithout,
    Label,
    Value,
    Arabic,
    RomanLower,
    RomanUpper,
    AlphLower,
    AlphUpper,
    Fnsymbol,
    NewLength,
    SetToWidth,
    SetToHeight,
    SetToDepth,
    DefineKey,
    SetKeys,
    /// `\verb` (reads raw characters from the source).
    Verb,
    /// Internal: stop reading all input (`\end{document}`).
    StopInput,
}

#[derive(Debug, Clone, PartialEq)]
enum SaveItem {
    CsMeaning(String, Meaning),
    Catcode(char, CatCode),
    Uccode(char, char),
    Lccode(char, char),
    IntParam(IntParam, i64),
    Count(u16, i64),
    Dimen(u16, i64),
    Skip(u16, Glue),
    Toks(u16, Vec<Token>),
}

#[derive(Debug, Clone, PartialEq)]
struct Frame {
    saves: Vec<SaveItem>,
    after_group: Vec<Token>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scopes {
    cs: HashMap<String, Meaning>,
    active: HashMap<char, Meaning>,
    cat_table: CatCodeTable,
    uccode: HashMap<char, char>,
    lccode: HashMap<char, char>,
    int_params: HashMap<IntParam, i64>,
    count: HashMap<u16, i64>,
    dimen: HashMap<u16, i64>,
    skip: HashMap<u16, Glue>,
    toks: HashMap<u16, Vec<Token>>,
    frames: Vec<Frame>,
}

impl Scopes {
    pub fn new() -> Self {
        let mut int_params = HashMap::new();
        int_params.insert(IntParam::Escapechar, '\\' as i64);
        int_params.insert(IntParam::Endlinechar, 13);
        int_params.insert(IntParam::Newlinechar, -1);
        int_params.insert(IntParam::ETeXVersion, 2);
        Scopes {
            cs: HashMap::new(),
            active: HashMap::new(),
            cat_table: CatCodeTable::latex_initial(),
            uccode: HashMap::new(),
            lccode: HashMap::new(),
            int_params,
            count: HashMap::new(),
            dimen: HashMap::new(),
            skip: HashMap::new(),
            toks: HashMap::new(),
            frames: vec![Frame { saves: Vec::new(), after_group: Vec::new() }],
        }
    }

    // -- control sequences --------------------------------------------

    pub fn meaning(&self, name: &str) -> Meaning {
        self.cs.get(name).cloned().unwrap_or(Meaning::Undefined)
    }

    pub fn meaning_ref(&self, name: &str) -> Option<&Meaning> {
        self.cs.get(name)
    }

    /// Is `name` currently defined (anything but `undefined`)? A group
    /// end can restore an entry *to* `Undefined`, so presence in the map
    /// is not enough (e-TeX `\ifcsname`, LaTeX `\@ifundefined`).
    pub fn is_defined(&self, name: &str) -> bool {
        !matches!(self.cs.get(name), None | Some(Meaning::Undefined))
    }

    pub fn active_meaning(&self, c: char) -> Meaning {
        self.active.get(&c).cloned().unwrap_or(Meaning::Undefined)
    }

    fn saving(&self) -> bool {
        // The outermost frame is never popped, so save entries there
        // would only ever accumulate; skip them.
        self.frames.len() > 1
    }

    pub fn assign_cs(&mut self, name: &str, meaning: Meaning, global: bool) {
        if !global && self.saving() {
            let previous = self.cs.get(name).cloned().unwrap_or(Meaning::Undefined);
            self.frames.last_mut().unwrap().saves.push(SaveItem::CsMeaning(name.to_string(), previous));
        }
        if global {
            // TeX's `\global` assignment also wipes any pending local
            // save entries for the same name? No -- TeX keeps them
            // (eq_destroy), but their restoration at group end is
            // suppressed only when the *saved* level marks it global.
            // We model that by restoring nothing special: a later group
            // end restores the local pre-assignment value, exactly like
            // TeX's behaviour for `{\def\a{1}\global\def\a{2}}` where
            // `\a` is `2` afterwards because TeX's `unsave` skips
            // restoring entries whose current level is `level_one`.
            self.retain_global_marker(name);
        }
        self.cs.insert(name.to_string(), meaning);
    }

    /// TeX rule (tex.web §283, `unsave`): once a control sequence has
    /// been assigned `\global`ly, save-stack entries for it made *in the
    /// current group before* that global assignment are discarded rather
    /// than restored. We implement that by dropping those entries now.
    fn retain_global_marker(&mut self, name: &str) {
        for frame in self.frames.iter_mut().skip(1) {
            frame.saves.retain(|s| !matches!(s, SaveItem::CsMeaning(n, _) if n == name));
        }
    }

    pub fn assign_active(&mut self, c: char, meaning: Meaning, global: bool) {
        // Active characters share the same save mechanism, keyed under a
        // synthetic name so one Vec<SaveItem> variant suffices.
        let key = format!("~active~{c}");
        if !global && self.saving() {
            let previous = self.active.get(&c).cloned().unwrap_or(Meaning::Undefined);
            self.frames.last_mut().unwrap().saves.push(SaveItem::CsMeaning(key, previous));
        } else {
            self.retain_global_marker(&key);
        }
        self.active.insert(c, meaning);
    }

    fn restore_cs_or_active(&mut self, key: String, meaning: Meaning) {
        if let Some(c) = key.strip_prefix("~active~").and_then(|s| s.chars().next()) {
            self.active.insert(c, meaning);
        } else {
            self.cs.insert(key, meaning);
        }
    }

    // -- catcodes / uccode / lccode / integer parameters -----------------

    pub fn catcode(&self, c: char) -> CatCode {
        self.cat_table.get(c)
    }

    pub fn set_catcode(&mut self, c: char, cat: CatCode, global: bool) {
        if !global && self.saving() {
            self.frames.last_mut().unwrap().saves.push(SaveItem::Catcode(c, self.cat_table.get(c)));
        }
        self.cat_table.set(c, cat);
    }

    pub fn cat_table(&self) -> &CatCodeTable {
        &self.cat_table
    }

    /// `\uccode`: INITEX sets uccode of letters to their uppercase form
    /// and everything else to 0 (TeXbook p. 41).
    pub fn uccode(&self, c: char) -> char {
        if let Some(u) = self.uccode.get(&c) {
            return *u;
        }
        if c.is_ascii_lowercase() {
            c.to_ascii_uppercase()
        } else if c.is_ascii_uppercase() {
            c
        } else {
            '\0'
        }
    }

    pub fn lccode(&self, c: char) -> char {
        if let Some(l) = self.lccode.get(&c) {
            return *l;
        }
        if c.is_ascii_uppercase() {
            c.to_ascii_lowercase()
        } else if c.is_ascii_lowercase() {
            c
        } else {
            '\0'
        }
    }

    pub fn set_uccode(&mut self, c: char, v: char, global: bool) {
        if !global && self.saving() {
            let old = self.uccode(c);
            self.frames.last_mut().unwrap().saves.push(SaveItem::Uccode(c, old));
        }
        self.uccode.insert(c, v);
    }

    pub fn set_lccode(&mut self, c: char, v: char, global: bool) {
        if !global && self.saving() {
            let old = self.lccode(c);
            self.frames.last_mut().unwrap().saves.push(SaveItem::Lccode(c, old));
        }
        self.lccode.insert(c, v);
    }

    pub fn int_param(&self, p: IntParam) -> i64 {
        *self.int_params.get(&p).unwrap_or(&0)
    }

    pub fn set_int_param(&mut self, p: IntParam, v: i64, global: bool) {
        if !global && self.saving() {
            let old = self.int_param(p);
            self.frames.last_mut().unwrap().saves.push(SaveItem::IntParam(p, old));
        }
        self.int_params.insert(p, v);
    }

    // -- registers ---------------------------------------------------------

    pub fn count(&self, idx: u16) -> i64 {
        *self.count.get(&idx).unwrap_or(&0)
    }
    pub fn set_count(&mut self, idx: u16, v: i64, global: bool) {
        if !global && self.saving() {
            let old = self.count(idx);
            self.frames.last_mut().unwrap().saves.push(SaveItem::Count(idx, old));
        } else {
            for frame in self.frames.iter_mut().skip(1) {
                frame.saves.retain(|s| !matches!(s, SaveItem::Count(i, _) if *i == idx));
            }
        }
        self.count.insert(idx, v);
    }
    pub fn dimen(&self, idx: u16) -> i64 {
        *self.dimen.get(&idx).unwrap_or(&0)
    }
    pub fn set_dimen(&mut self, idx: u16, v: i64, global: bool) {
        if !global && self.saving() {
            let old = self.dimen(idx);
            self.frames.last_mut().unwrap().saves.push(SaveItem::Dimen(idx, old));
        } else {
            for frame in self.frames.iter_mut().skip(1) {
                frame.saves.retain(|s| !matches!(s, SaveItem::Dimen(i, _) if *i == idx));
            }
        }
        self.dimen.insert(idx, v);
    }
    pub fn skip(&self, idx: u16) -> Glue {
        *self.skip.get(&idx).unwrap_or(&Glue::fixed(0))
    }
    pub fn set_skip(&mut self, idx: u16, v: Glue, global: bool) {
        if !global && self.saving() {
            let old = self.skip(idx);
            self.frames.last_mut().unwrap().saves.push(SaveItem::Skip(idx, old));
        } else {
            for frame in self.frames.iter_mut().skip(1) {
                frame.saves.retain(|s| !matches!(s, SaveItem::Skip(i, _) if *i == idx));
            }
        }
        self.skip.insert(idx, v);
    }
    pub fn toks(&self, idx: u16) -> Vec<Token> {
        self.toks.get(&idx).cloned().unwrap_or_default()
    }
    pub fn set_toks(&mut self, idx: u16, v: Vec<Token>, global: bool) {
        if !global && self.saving() {
            let old = self.toks(idx);
            self.frames.last_mut().unwrap().saves.push(SaveItem::Toks(idx, old));
        } else {
            for frame in self.frames.iter_mut().skip(1) {
                frame.saves.retain(|s| !matches!(s, SaveItem::Toks(i, _) if *i == idx));
            }
        }
        self.toks.insert(idx, v);
    }

    // -- grouping ------------------------------------------------------

    pub fn push_group(&mut self) {
        self.frames.push(Frame { saves: Vec::new(), after_group: Vec::new() });
    }

    pub fn pop_group(&mut self) -> Vec<Token> {
        if self.frames.len() <= 1 {
            return Vec::new();
        }
        let frame = self.frames.pop().unwrap();
        for item in frame.saves.into_iter().rev() {
            match item {
                SaveItem::CsMeaning(name, m) => self.restore_cs_or_active(name, m),
                SaveItem::Catcode(c, cat) => self.cat_table.set(c, cat),
                SaveItem::Uccode(c, v) => {
                    self.uccode.insert(c, v);
                }
                SaveItem::Lccode(c, v) => {
                    self.lccode.insert(c, v);
                }
                SaveItem::IntParam(p, v) => {
                    self.int_params.insert(p, v);
                }
                SaveItem::Count(idx, v) => {
                    self.count.insert(idx, v);
                }
                SaveItem::Dimen(idx, v) => {
                    self.dimen.insert(idx, v);
                }
                SaveItem::Skip(idx, v) => {
                    self.skip.insert(idx, v);
                }
                SaveItem::Toks(idx, v) => {
                    self.toks.insert(idx, v);
                }
            }
        }
        frame.after_group
    }

    pub fn queue_aftergroup(&mut self, tok: Token) {
        self.frames.last_mut().unwrap().after_group.push(tok);
    }

    pub fn depth(&self) -> usize {
        self.frames.len()
    }

    /// Iterate every stored token (macro bodies, toks registers, saved
    /// meanings, `\aftergroup` queues) -- used by the incremental
    /// expander's span-aware state comparison.
    pub fn for_each_meaning<'s>(&'s self, mut f: impl FnMut(&'s Meaning)) {
        for m in self.cs.values() {
            f(m);
        }
        for m in self.active.values() {
            f(m);
        }
        for frame in &self.frames {
            for s in &frame.saves {
                if let SaveItem::CsMeaning(_, m) = s {
                    f(m);
                }
            }
        }
    }
}

impl Default for Scopes {
    fn default() -> Self {
        Self::new()
    }
}

// ---- span mapping (incremental convergence) ---------------------------

fn map_token(t: &Token, f: &dyn Fn(Span) -> Option<Span>) -> Option<Token> {
    Some(Token::new(t.kind.clone(), f(t.span)?))
}

fn map_tokens(ts: &[Token], f: &dyn Fn(Span) -> Option<Span>) -> Option<Vec<Token>> {
    ts.iter().map(|t| map_token(t, f)).collect()
}

fn map_macro(d: &MacroDef, f: &dyn Fn(Span) -> Option<Span>) -> Option<MacroDef> {
    use crate::macro_def::{BodyPart, ParamPart};
    let params = d
        .params
        .iter()
        .map(|p| match p {
            ParamPart::Literal(t) => map_token(t, f).map(ParamPart::Literal),
            ParamPart::Param(n) => Some(ParamPart::Param(*n)),
        })
        .collect::<Option<Vec<_>>>()?;
    let body = d
        .body
        .iter()
        .map(|p| match p {
            BodyPart::Literal(t) => map_token(t, f).map(BodyPart::Literal),
            BodyPart::Param(n) => Some(BodyPart::Param(*n)),
        })
        .collect::<Option<Vec<_>>>()?;
    Some(MacroDef { params, body, flags: d.flags, arity: d.arity })
}

pub(crate) fn map_meaning(m: &Meaning, f: &dyn Fn(Span) -> Option<Span>) -> Option<Meaning> {
    Some(match m {
        Meaning::Macro(d) => Meaning::Macro(Rc::new(map_macro(d, f)?)),
        Meaning::Let(inner) => Meaning::Let(Box::new(map_meaning(inner, f)?)),
        Meaning::CharLike(t) => Meaning::CharLike(map_token(t, f)?),
        other => other.clone(),
    })
}

impl Scopes {
    /// Rebuild this state with every stored token span passed through
    /// `f`; `None` if any span is rejected. Used to compare an old
    /// checkpoint's state against a new run's after an edit shifted the
    /// source.
    pub fn map_spans(&self, f: &dyn Fn(Span) -> Option<Span>) -> Option<Scopes> {
        let cs = self
            .cs
            .iter()
            .map(|(k, v)| map_meaning(v, f).map(|m| (k.clone(), m)))
            .collect::<Option<HashMap<_, _>>>()?;
        let active = self
            .active
            .iter()
            .map(|(k, v)| map_meaning(v, f).map(|m| (*k, m)))
            .collect::<Option<HashMap<_, _>>>()?;
        let toks = self
            .toks
            .iter()
            .map(|(k, v)| map_tokens(v, f).map(|t| (*k, t)))
            .collect::<Option<HashMap<_, _>>>()?;
        let frames = self
            .frames
            .iter()
            .map(|fr| {
                let saves = fr
                    .saves
                    .iter()
                    .map(|s| {
                        Some(match s {
                            SaveItem::CsMeaning(n, m) => SaveItem::CsMeaning(n.clone(), map_meaning(m, f)?),
                            SaveItem::Toks(i, v) => SaveItem::Toks(*i, map_tokens(v, f)?),
                            other => other.clone(),
                        })
                    })
                    .collect::<Option<Vec<_>>>()?;
                Some(Frame { saves, after_group: map_tokens(&fr.after_group, f)? })
            })
            .collect::<Option<Vec<_>>>()?;
        Some(Scopes {
            cs,
            active,
            cat_table: self.cat_table.clone(),
            uccode: self.uccode.clone(),
            lccode: self.lccode.clone(),
            int_params: self.int_params.clone(),
            count: self.count.clone(),
            dimen: self.dimen.clone(),
            skip: self.skip.clone(),
            toks,
            frames,
        })
    }
}
