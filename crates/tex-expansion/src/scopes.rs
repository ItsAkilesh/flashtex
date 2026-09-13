//! Unified grouping/save-stack for everything TeX assignments can touch
//! locally: control-sequence meanings (`\def`/`\let`/...), category codes
//! (`\catcode`), and registers (`\count`/`\dimen`/`\skip`/`\toks`).
//! TeXbook ch. 24's "save stack": `{`/`}` and `\begingroup`/`\endgroup`
//! open/close a scope; assignments are local by default and are undone
//! when their scope closes; `\global` makes them permanent (no save
//! entry is pushed) and `\aftergroup` queues a token for reinsertion right
//! after the scope closes.

use std::collections::HashMap;
use std::rc::Rc;

use crate::catcode::{CatCode, CatCodeTable};
use crate::macro_def::MacroDef;
use crate::registers::Glue;
use crate::token::Token;

#[derive(Debug, Clone)]
pub enum Meaning {
    Macro(Rc<MacroDef>),
    /// A `\newcommand`/`\newenvironment`-style macro whose first parameter
    /// is optional (`\newcommand\foo[2][default]{...}`): `body` has arity
    /// N with `#1` bound either to the bracketed `[...]` given at the call
    /// site or to `default` when the caller omits it.
    MacroWithOptional { body: Rc<MacroDef>, default: Vec<Token> },
    Let(Box<Meaning>),
    CharLike(Token),
    Primitive(Primitive),
    RegisterAlias(RegisterKind, u16),
    Undefined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterKind {
    Count,
    Dimen,
    Skip,
    Toks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Primitive {
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
    Expandafter,
    Noexpand,
    Csname,
    Endcsname,
    String,
    Number,
    Romannumeral,
    The,
    Begingroup,
    Endgroup,
    Aftergroup,
    Catcode,
    Makeatletter,
    Makeatother,
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
    Advance,
    Multiply,
    Divide,
    Numexpr,
    Dimexpr,
    // -- LaTeX layer (built on the primitives above) --
    NewCommand,
    RenewCommand,
    ProvideCommand,
    NewEnvironment,
    RenewEnvironment,
    Begin,
    End,
    NewCounter,
    SetCounter,
    AddToCounter,
    StepCounter,
    RefStepCounter,
    Value,
    Arabic,
    RomanLower,
    RomanUpper,
    AlphLower,
    AlphUpper,
    Fnsymbol,
    IfNextChar,
    IfStar,
    NameDef,
    NameUse,
}

enum SaveItem {
    CsMeaning(String, Meaning),
    Catcode(char, CatCode),
    Count(u16, i64),
    Dimen(u16, i64),
    Skip(u16, Glue),
    Toks(u16, Vec<Token>),
}

struct Frame {
    saves: Vec<SaveItem>,
    after_group: Vec<Token>,
}

pub struct Scopes {
    cs: HashMap<String, Meaning>,
    active: HashMap<char, Meaning>,
    cat_table: CatCodeTable,
    count: HashMap<u16, i64>,
    dimen: HashMap<u16, i64>,
    skip: HashMap<u16, Glue>,
    toks: HashMap<u16, Vec<Token>>,
    frames: Vec<Frame>,
}

impl Scopes {
    pub fn new() -> Self {
        Scopes {
            cs: HashMap::new(),
            active: HashMap::new(),
            cat_table: CatCodeTable::latex_initial(),
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

    pub fn active_meaning(&self, c: char) -> Meaning {
        self.active.get(&c).cloned().unwrap_or(Meaning::Undefined)
    }

    pub fn assign_cs(&mut self, name: &str, meaning: Meaning, global: bool) {
        if !global {
            let previous = self.cs.get(name).cloned().unwrap_or(Meaning::Undefined);
            self.frames.last_mut().unwrap().saves.push(SaveItem::CsMeaning(name.to_string(), previous));
        }
        self.cs.insert(name.to_string(), meaning);
    }

    pub fn assign_active(&mut self, c: char, meaning: Meaning, global: bool) {
        // Active characters share the same save mechanism, keyed under a
        // synthetic name so one Vec<SaveItem> variant suffices.
        if !global {
            let previous = self.active.get(&c).cloned().unwrap_or(Meaning::Undefined);
            self.frames.last_mut().unwrap().saves.push(SaveItem::CsMeaning(format!("~active~{c}"), previous));
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

    // -- catcodes --------------------------------------------------------

    pub fn catcode(&self, c: char) -> CatCode {
        self.cat_table.get(c)
    }

    pub fn set_catcode(&mut self, c: char, cat: CatCode, global: bool) {
        if !global {
            self.frames.last_mut().unwrap().saves.push(SaveItem::Catcode(c, self.cat_table.get(c)));
        }
        self.cat_table.set(c, cat);
    }

    pub fn cat_table(&self) -> &CatCodeTable {
        &self.cat_table
    }

    // -- registers ---------------------------------------------------------

    pub fn count(&self, idx: u16) -> i64 {
        *self.count.get(&idx).unwrap_or(&0)
    }
    pub fn set_count(&mut self, idx: u16, v: i64, global: bool) {
        if !global {
            let old = self.count(idx);
            self.frames.last_mut().unwrap().saves.push(SaveItem::Count(idx, old));
        }
        self.count.insert(idx, v);
    }
    pub fn dimen(&self, idx: u16) -> i64 {
        *self.dimen.get(&idx).unwrap_or(&0)
    }
    pub fn set_dimen(&mut self, idx: u16, v: i64, global: bool) {
        if !global {
            let old = self.dimen(idx);
            self.frames.last_mut().unwrap().saves.push(SaveItem::Dimen(idx, old));
        }
        self.dimen.insert(idx, v);
    }
    pub fn skip(&self, idx: u16) -> Glue {
        *self.skip.get(&idx).unwrap_or(&Glue::fixed(0))
    }
    pub fn set_skip(&mut self, idx: u16, v: Glue, global: bool) {
        if !global {
            let old = self.skip(idx);
            self.frames.last_mut().unwrap().saves.push(SaveItem::Skip(idx, old));
        }
        self.skip.insert(idx, v);
    }
    pub fn toks(&self, idx: u16) -> Vec<Token> {
        self.toks.get(&idx).cloned().unwrap_or_default()
    }
    pub fn set_toks(&mut self, idx: u16, v: Vec<Token>, global: bool) {
        if !global {
            let old = self.toks(idx);
            self.frames.last_mut().unwrap().saves.push(SaveItem::Toks(idx, old));
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
}

impl Default for Scopes {
    fn default() -> Self {
        Self::new()
    }
}
