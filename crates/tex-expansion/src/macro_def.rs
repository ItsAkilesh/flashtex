use crate::token::Token;

/// One element of a macro's parameter text (TeXbook ch. 20): either a
/// literal token that must match verbatim (delimited parameters), or a
/// parameter slot `#1`..`#9`.
#[derive(Debug, Clone, PartialEq)]
pub enum ParamPart {
    Literal(Token),
    Param(u8),
}

/// One element of a macro's replacement text: a literal token, or a
/// reference to argument `#1..#9` substituted at call time.
#[derive(Debug, Clone, PartialEq)]
pub enum BodyPart {
    Literal(Token),
    Param(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroFlags {
    pub long: bool,
    pub outer: bool,
    /// `#{` form: the last parameter is delimited by an immediately
    /// following `{`, which is *not* consumed (TeXbook p. 205).
    pub brace_delimited_last: bool,
}

impl Default for MacroFlags {
    fn default() -> Self {
        MacroFlags { long: false, outer: false, brace_delimited_last: false }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MacroDef {
    pub params: Vec<ParamPart>,
    pub body: Vec<BodyPart>,
    pub flags: MacroFlags,
    /// Number of `#1..#9` parameters (highest index referenced).
    pub arity: u8,
}

impl MacroDef {
    pub fn param_count(&self) -> u8 {
        self.arity
    }
}
