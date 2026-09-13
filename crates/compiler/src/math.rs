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
/// Symbol.afm `radical` (C 214): ink right edge 515 and top 917, per 1000 em.
pub const RADICAL_INK_RIGHT_EM: f64 = 0.515;
pub const RADICAL_TOP_EM: f64 = 0.917;
/// Symbol.afm `radicalex` (C 96), the vinculum extender: y 881..917.
pub const RADICALEX_THICKNESS_EM: f64 = 0.036;
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
    /// Forces this atom's TeXbook Chapter 17 spacing class rather than
    /// deriving it from the nucleus (see `atom_class`).
    ///
    /// Needed whenever the same glyph must carry two different classes
    /// depending on which command produced it (`\bot` is Ord where `\perp`'s
    /// identical U+22A5 glyph is Rel; `\bigtriangleup` is Bin where
    /// `\triangle`'s identical U+25B3 glyph is Ord), and by the
    /// `\mathbin`/`\mathrel`/`\mathord`/`\mathop`/`\mathopen`/`\mathclose`/
    /// `\mathpunct` family, which boxes an arbitrary math list as one atom of
    /// the stated class.
    pub(crate) class_override: Option<AtomClass>,
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
    /// `\mathbf{...}`: literal text in the bold roman face.
    Bold(String),
    /// `\boxed`, `\overline` and `\underline`: a list with real rules.
    Framed {
        body: MathList,
        frame: Frame,
    },
    /// `\overset`, `\underset`, `\stackrel`: a base with a script-size list
    /// centred directly above or below it.
    Stacked {
        base: MathList,
        over: Option<MathList>,
        under: Option<MathList>,
    },
    /// `array`, `cases` and the amsmath matrix environments: a grid of cells
    /// with per-column alignment (`l`, `c`, `r`) and optional stretched fences.
    Matrix {
        rows: Vec<Vec<MathList>>,
        columns: String,
        left: String,
        right: String,
    },
    /// `\hat`, `\bar`, `\vec`, ..., `\widehat`, `\widetilde`: a mark placed
    /// over `body`. See [`Accent`] for which marks have a real base-14 glyph.
    Accent {
        accent: Accent,
        body: MathList,
    },
    /// `\mathbin{...}`, `\mathrel{...}`, and the rest of the `\math*` class
    /// family (TeXbook Chapter 17): an arbitrary math list boxed as a single
    /// atom, laid out like a bare `{...}` group. The enclosing [`MathAtom`]'s
    /// `class_override` carries the forced spacing class; this variant only
    /// exists so a multi-atom argument stays one atom for spacing purposes
    /// instead of flattening into the surrounding list.
    Group(MathList),
}

/// `\hat`..`\grave`, plus `\widehat`/`\widetilde`.
///
/// The compiler renders math with Adobe's Core 14 Symbol/Times-Roman faces,
/// not Computer Modern, so TeX's exact accent geometry is not reproducible.
/// Where a real base-14 glyph exists for the
/// mark, it is used, scaled and centered over `body`; `\check` and `\breve`
/// have no such glyph (no caron or breve character in WinAnsi or the Symbol
/// encoding — see `crate::export`) and are reported rather than faked, the
/// same policy `crate::export::map_char` already applies to every other
/// unrepresentable character.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Accent {
    Hat,
    Bar,
    Vec,
    Tilde,
    Dot,
    Ddot,
    Check,
    Breve,
    Acute,
    Grave,
    WideHat,
    WideTilde,
}

impl Accent {
    pub fn command(self) -> &'static str {
        match self {
            Accent::Hat => "hat",
            Accent::Bar => "bar",
            Accent::Vec => "vec",
            Accent::Tilde => "tilde",
            Accent::Dot => "dot",
            Accent::Ddot => "ddot",
            Accent::Check => "check",
            Accent::Breve => "breve",
            Accent::Acute => "acute",
            Accent::Grave => "grave",
            Accent::WideHat => "widehat",
            Accent::WideTilde => "widetilde",
        }
    }

    /// The glyph drawn above `body`, or `None` when no base-14 glyph exists.
    ///
    /// `\widehat`/`\widetilde` reuse the plain `\hat`/`\tilde` glyph: TeX
    /// grows these from a cmex10 successor chain to cover a wide base, and
    /// there is no equivalent stretchy glyph or font-growing mechanism here,
    /// so a multi-atom base gets a diagnostic (see `accent_atom`) rather than
    /// a silently-too-narrow mark.
    pub fn glyph(self) -> Option<char> {
        match self {
            Accent::Hat | Accent::WideHat => Some('\u{2C6}'), // circumflex accent
            Accent::Bar => Some('\u{AF}'),                    // macron
            // TeX's \vec draws a short low arrow; the closest real base-14
            // glyph is the full-size Symbol arrowright. An approximation,
            // not a fabrication: it is a real arrow glyph, just not the
            // exact short accent stroke.
            Accent::Vec => Some('\u{2192}'),
            Accent::Tilde | Accent::WideTilde => Some('\u{2DC}'), // small tilde
            // TeX's \dot is a raised dot above (U+02D9), which has no
            // base-14 glyph either. The Symbol/Times middle dot U+00B7 (the
            // same character already used for \cdot) is the closest real
            // stand-in.
            Accent::Dot => Some('\u{B7}'),
            Accent::Ddot => Some('\u{A8}'),  // diaeresis
            Accent::Acute => Some('\u{B4}'), // acute accent
            Accent::Grave => Some('\u{60}'), // grave accent
            Accent::Check | Accent::Breve => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Frame {
    Box,
    Over,
    Under,
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
    ("alignedat", 'c', "", ""),
    ("split", 'c', "", ""),
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
        pending: Vec::new(),
    }
    .list(false)
}

/// Maximum nesting of braced math groups, scripts, fractions and radicals.
///
/// The list parser is recursive descent, so a document full of unclosed openers
/// recurses once per opener. Without a bound, a pathological file — or a
/// half-typed one — overflows the stack and kills the worker mid-keystroke.
/// Exceeding the bound is an explicit diagnostic, not a crash.
// Keep ample headroom for the command parser's stack frame on the macOS Swift
// app's worker thread as the supported command set grows. The previous 256
// limit could exhaust that thread before the guard was reached.
pub const MAX_MATH_DEPTH: usize = 128;

struct MathParser<'a> {
    tokens: &'a [Token],
    i: usize,
    depth: usize,
    diagnostics: &'a mut Vec<Diagnostic>,
    /// Atoms produced by the last `atom()` call beyond the one it returned
    /// (a flattened style group, a root index), in order after it.
    pending: Vec<MathAtom>,
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
                // Limit-placement switches produce no atom, so a following
                // script still attaches to the operator (`\lim\limits_{x}`).
                TokenKind::Command(ref switch) if switch == "limits" || switch == "nolimits" => {
                    self.i += 1;
                }
                TokenKind::Command(ref infix) if infix == "choose" || infix == "over" => {
                    // TeX infix forms: everything before in this group is the
                    // top, everything after (to the group's end) the bottom.
                    self.i += 1;
                    let top = MathList {
                        atoms: std::mem::take(&mut atoms),
                    };
                    let bottom = self.list(stop_at_brace);
                    let nucleus = if infix == "over" {
                        Nucleus::Fraction {
                            numerator: top,
                            denominator: bottom,
                        }
                    } else {
                        Nucleus::Matrix {
                            rows: vec![vec![top], vec![bottom]],
                            columns: "c".into(),
                            left: "(".into(),
                            right: ")".into(),
                        }
                    };
                    return MathList {
                        atoms: vec![MathAtom {
                            nucleus,
                            span: token.span,
                            superscript: None,
                            subscript: None,
                            class_override: None,
                        }],
                    };
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
                        atoms.append(&mut self.pending);
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
            let mut atoms = vec![atom];
            atoms.append(&mut self.pending);
            MathList { atoms }
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
                        ':' | '>' => 4.0,
                        ';' => 5.0,
                        ' ' => 6.0,
                        '!' => -3.0,
                        _ => 0.0,
                    };
                    if mu != 0.0 {
                        return Some(space(mu / 18.0, token.span));
                    }
                    if ch == '|' {
                        return Some(symbol("∣∣".into(), token.span));
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
        if let Some(operator) = OPERATOR_NAMES.iter().find(|op| **op == name) {
            return text_atom(operator.to_string(), span);
        }
        match name.as_str() {
            // Plain TeX's `\iff` and mathtools's `\implies`/`\impliedby` are
            // macros that expand to a thick space (`\;`, 5mu), the long
            // double arrow, and another thick space — not a bare glyph — so
            // they need their own arms rather than a `COMMAND_GLYPHS` row.
            "iff" | "implies" | "impliedby" => {
                let arrow = match name.as_str() {
                    "iff" => "⟺",
                    "implies" => "⟹",
                    _ => "⟸",
                };
                self.pending.push(symbol(arrow.into(), span));
                self.pending.push(space(5.0 / 18.0, span));
                space(5.0 / 18.0, span)
            }
            // `\bot` renders the exact same Symbol glyph as `\perp`
            // (U+22A5), but is Ord where `\perp` is Rel; `symbol_class` is
            // keyed by glyph, so the class must be forced on the atom instead
            // of invented as a second glyph.
            "bot" => MathAtom {
                class_override: Some(AtomClass::Ord),
                ..symbol("⊥".into(), span)
            },
            // `\bigtriangleup` renders `\triangle`'s exact glyph (U+25B3) but
            // is Bin where `\triangle` is Ord; same fix as `\bot`/`\perp`.
            "bigtriangleup" => MathAtom {
                class_override: Some(AtomClass::Bin),
                ..symbol("△".into(), span)
            },
            // TeXbook Chapter 17's `\mathbin`/`\mathrel`/... family: the
            // argument is a full math list, boxed as one atom whose class is
            // forced regardless of what its own contents would imply.
            "mathbin" | "mathrel" | "mathord" | "mathop" | "mathopen" | "mathclose"
            | "mathpunct" => {
                let class = match name.as_str() {
                    "mathbin" => AtomClass::Bin,
                    "mathrel" => AtomClass::Rel,
                    "mathop" => AtomClass::Op,
                    "mathopen" => AtomClass::Open,
                    "mathclose" => AtomClass::Close,
                    "mathpunct" => AtomClass::Punct,
                    _ => AtomClass::Ord,
                };
                let body = self.required_group(&name, span);
                MathAtom {
                    nucleus: Nucleus::Group(body),
                    span,
                    superscript: None,
                    subscript: None,
                    class_override: Some(class),
                }
            }
            "operatorname" => {
                self.skip_star();
                let (text, argument_span) = self.required_text_group("operatorname", span);
                text_atom(text, span.merge(argument_span))
            }
            // Upright roman is already the math default in this subset, and
            // the other style switches have no distinct face yet: keep the
            // argument's content rather than dropping or garbling it.
            "mathrm" | "mathit" | "mathsf" | "mathtt" | "mathnormal" | "boldsymbol" | "bm"
            | "mbox" | "hbox" | "textrm" | "textit" | "textnormal" => {
                let body = self.required_group(&name, span);
                self.group_atom(body, span)
            }
            "displaystyle" | "textstyle" | "scriptstyle" | "scriptscriptstyle" | "nonumber"
            | "notag" | "middle" => space(0.0, span),
            "left" | "right" | "big" | "Big" | "bigg" | "Bigg" | "bigm" | "Bigm" | "biggm"
            | "Biggm" | "Bigl" | "Bigr" | "biggl" | "biggr" | "Biggl" | "Biggr" => {
                self.take_delimiter(&name, span)
            }
            "dots" | "ldots" | "dotsc" | "dotso" => text_atom("...".into(), span),
            "cdots" | "dotsb" | "dotsm" | "dotsi" => symbol("⋅⋅⋅".into(), span),
            // Symbol has no U+222C/U+222D: repeated real integral glyphs.
            "iint" => symbol("∫∫".into(), span),
            "lbrace" => symbol("{".into(), span),
            "rbrace" => symbol("}".into(), span),
            "iiint" => symbol("∫∫∫".into(), span),
            "bmod" | "mod" => text_atom("mod".into(), span),
            "dfrac" | "tfrac" | "cfrac" => self.command_atom("frac".into(), span),
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
                    class_override: None,
                }
            }
            "begin" => self.grid_environment(span),
            "sqrt" => {
                let index = self.optional_bracket_list();
                let radical = MathAtom {
                    nucleus: Nucleus::Radical(self.required_group("sqrt", span)),
                    span,
                    superscript: None,
                    subscript: None,
                    class_override: None,
                };
                match index {
                    // The root index sits as a raised script ahead of the sign.
                    Some(index) if !index.atoms.is_empty() => {
                        self.pending.push(radical);
                        MathAtom {
                            superscript: Some(index),
                            ..space(0.0, span)
                        }
                    }
                    _ => radical,
                }
            }
            "overset" | "stackrel" | "underset" => {
                let script = self.required_group(&name, span);
                let base = self.required_group(&name, span);
                let (over, under) = if name == "underset" {
                    (None, Some(script))
                } else {
                    (Some(script), None)
                };
                MathAtom {
                    nucleus: Nucleus::Stacked { base, over, under },
                    span,
                    superscript: None,
                    subscript: None,
                    class_override: None,
                }
            }
            "binom" | "dbinom" | "tbinom" => {
                let top = self.required_group(&name, span);
                let bottom = self.required_group(&name, span);
                MathAtom {
                    nucleus: Nucleus::Matrix {
                        rows: vec![vec![top], vec![bottom]],
                        columns: "c".into(),
                        left: "(".into(),
                        right: ")".into(),
                    },
                    span,
                    superscript: None,
                    subscript: None,
                    class_override: None,
                }
            }
            "mathbf" | "textbf" => {
                let (text, argument_span) = self.required_text_group(&name, span);
                MathAtom {
                    nucleus: Nucleus::Bold(text),
                    span: span.merge(argument_span),
                    superscript: None,
                    subscript: None,
                    class_override: None,
                }
            }
            "boxed" | "overline" | "underline" => {
                let body = self.required_group(&name, span);
                let frame = match name.as_str() {
                    "boxed" => Frame::Box,
                    "overline" => Frame::Over,
                    _ => Frame::Under,
                };
                MathAtom {
                    nucleus: Nucleus::Framed { body, frame },
                    span,
                    superscript: None,
                    subscript: None,
                    class_override: None,
                }
            }
            "tag" => {
                let starred = matches!(self.tokens.get(self.i).map(|t| &t.kind), Some(TokenKind::Word(w)) if w == "*");
                self.skip_star();
                let (text, argument_span) = self.required_text_group("tag", span);
                let label = if starred { text } else { format!("({text})") };
                self.pending
                    .push(text_atom(label, span.merge(argument_span)));
                space(2.0 * QUAD_EM, span)
            }
            "pmod" => {
                let body = self.required_group("pmod", span);
                self.pending.push(text_atom("(mod".into(), span));
                self.pending.push(space(6.0 / 18.0, span));
                self.pending.extend(body.atoms);
                self.pending.push(text_atom(")".into(), span));
                space(QUAD_EM, span)
            }
            "text" => {
                let (text, argument_span) = self.required_text_group("text", span);
                MathAtom {
                    nucleus: Nucleus::Text(text),
                    span: span.merge(argument_span),
                    superscript: None,
                    subscript: None,
                    class_override: None,
                }
            }
            // Delimiter stretching is not implemented yet. Consume and retain
            // the requested delimiter at ordinary size instead of fabricating a
            // hard-coded parenthesis (which would duplicate the source token).
            "bigl" | "bigr" => self.take_delimiter(&name, span),
            "quad" => space(QUAD_EM, span),
            "qquad" => space(2.0 * QUAD_EM, span),
            "mathbb" => {
                let (text, argument_span) = self.required_text_group("mathbb", span);
                let span = span.merge(argument_span);
                let letters: String = text.chars().filter(|c| !c.is_whitespace()).collect();
                match letters
                    .chars()
                    .map(crate::lm_math::double_struck)
                    .collect::<Option<String>>()
                {
                    Some(glyphs) if !glyphs.is_empty() => symbol(glyphs, span),
                    _ => {
                        self.diagnostics.push(Diagnostic::error(
                            format!(
                                "\\mathbb supports only capital letters A-Z, not {:?}",
                                letters
                            ),
                            Some(span),
                            Some("typeset the argument without blackboard bold".into()),
                        ));
                        symbol(letters, span)
                    }
                }
            }
            "hat" => self.accent_atom(Accent::Hat, span),
            "bar" => self.accent_atom(Accent::Bar, span),
            "vec" => self.accent_atom(Accent::Vec, span),
            "tilde" => self.accent_atom(Accent::Tilde, span),
            "dot" => self.accent_atom(Accent::Dot, span),
            "ddot" => self.accent_atom(Accent::Ddot, span),
            "check" => self.accent_atom(Accent::Check, span),
            "breve" => self.accent_atom(Accent::Breve, span),
            "acute" => self.accent_atom(Accent::Acute, span),
            "grave" => self.accent_atom(Accent::Grave, span),
            "widehat" => self.accent_atom(Accent::WideHat, span),
            "widetilde" => self.accent_atom(Accent::WideTilde, span),
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

    /// Returns the first atom of `body` and queues the rest, so the group
    /// flattens into the surrounding list exactly like a bare `{...}` group.
    fn group_atom(&mut self, body: MathList, span: Span) -> MathAtom {
        let mut atoms = body.atoms.into_iter();
        match atoms.next() {
            Some(first) => {
                self.pending.extend(atoms);
                first
            }
            None => space(0.0, span),
        }
    }

    /// An optional `[...]` math argument, as in `\sqrt[n]{x}`.
    fn optional_bracket_list(&mut self) -> Option<MathList> {
        let mut cursor = self.i;
        while matches!(
            self.tokens.get(cursor).map(|t| &t.kind),
            Some(TokenKind::Space)
        ) {
            cursor += 1;
        }
        if !matches!(self.tokens.get(cursor).map(|t| &t.kind), Some(TokenKind::Word(w)) if w == "[")
        {
            return None;
        }
        let start = cursor + 1;
        let mut depth = 0usize;
        let mut end = start;
        while let Some(token) = self.tokens.get(end) {
            match &token.kind {
                TokenKind::LBrace => depth += 1,
                TokenKind::RBrace => depth = depth.saturating_sub(1),
                TokenKind::Word(w) if w == "]" && depth == 0 => break,
                _ => {}
            }
            end += 1;
        }
        if end >= self.tokens.len() {
            return None;
        }
        self.i = end + 1;
        Some(
            MathParser {
                tokens: &self.tokens[start..end],
                i: 0,
                depth: self.depth,
                diagnostics: self.diagnostics,
                pending: Vec::new(),
            }
            .list(false),
        )
    }

    fn skip_star(&mut self) {
        if matches!(self.tokens.get(self.i).map(|t| &t.kind), Some(TokenKind::Word(w)) if w == "*")
        {
            self.i += 1;
        }
    }

    fn accent_atom(&mut self, accent: Accent, span: Span) -> MathAtom {
        let body = self.required_group(accent.command(), span);
        if accent.glyph().is_none() {
            self.diagnostics.push(Diagnostic::warning(
                format!(
                    "\\{} has no representable accent glyph in the compiler's base-14 fonts",
                    accent.command()
                ),
                Some(span),
                Some("typeset the base without the accent mark and continued".into()),
            ));
        } else if matches!(accent, Accent::WideHat | Accent::WideTilde) && body.atoms.len() > 1 {
            self.diagnostics.push(Diagnostic::warning(
                format!(
                    "\\{} does not stretch to cover more than one symbol without a cmex-style growing glyph",
                    accent.command()
                ),
                Some(span),
                Some("centered a fixed-width accent glyph over the whole base and continued".into()),
            ));
        }
        MathAtom {
            nucleus: Nucleus::Accent { accent, body },
            span,
            superscript: None,
            subscript: None,
            class_override: None,
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
        if let TokenKind::Command(name) = &token.kind {
            let glyph = match name.as_str() {
                "lbrace" => Some("{"),
                "rbrace" => Some("}"),
                "vert" => Some("|"),
                "Vert" => Some("∣∣"),
                other => command_glyph(other).filter(|_| DELIMITER_COMMANDS.contains(&other)),
            };
            if let Some(glyph) = glyph {
                self.i += 1;
                return symbol(glyph.into(), span.merge(token.span));
            }
        }
        let TokenKind::Word(delimiter) = &token.kind else {
            self.diagnostics.push(Diagnostic::error(
                format!("\\{command} requires a following delimiter"),
                Some(span),
                Some("left the following non-delimiter token to be parsed normally".into()),
            ));
            return symbol(String::new(), span);
        };
        if delimiter == "." {
            // The null delimiter: an invisible fence (`\left.` / `\right.`).
            self.i += 1;
            return space(0.0, span.merge(token.span));
        }
        if delimiter == "|" && token.span.end - token.span.start == 2 {
            self.i += 1;
            return symbol("∣∣".into(), span.merge(token.span));
        }
        if delimiter.chars().count() != 1 || !"()[]{}|./<>".contains(delimiter.as_str()) {
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
        if name == "alignedat" {
            // The column-pair count argument; the grid sizes itself from cells.
            let _ = self.required_text_group("alignedat", span);
        }
        if matches!(name.as_str(), "aligned" | "alignedat" | "split") {
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
                            pending: Vec::new(),
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
            class_override: None,
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
        class_override: None,
    }
}

fn space(em: f64, span: Span) -> MathAtom {
    MathAtom {
        nucleus: Nucleus::Space { em },
        span,
        superscript: None,
        subscript: None,
        class_override: None,
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
    // Handwritten-homework coverage (Adobe Symbol encodes every glyph below).
    // Symbol has only the open-form epsilon (0x65), no lunate U+03F5, so
    // `\epsilon` shares `\varepsilon`'s glyph; the README states this.
    ("epsilon", "ε"),
    ("varepsilon", "ε"),
    ("zeta", "ζ"),
    ("eta", "η"),
    ("vartheta", "ϑ"),
    ("iota", "ι"),
    ("kappa", "κ"),
    ("nu", "ν"),
    ("xi", "ξ"),
    ("varpi", "ϖ"),
    ("rho", "ρ"),
    ("varsigma", "ς"),
    ("tau", "τ"),
    ("upsilon", "υ"),
    ("varphi", "ϕ"),
    ("chi", "χ"),
    ("psi", "ψ"),
    ("Gamma", "Γ"),
    ("Delta", "Δ"),
    ("Theta", "Θ"),
    ("Lambda", "Λ"),
    ("Xi", "Ξ"),
    ("Pi", "Π"),
    ("Sigma", "Σ"),
    ("Upsilon", "Υ"),
    ("Phi", "Φ"),
    ("Psi", "Ψ"),
    ("Omega", "Ω"),
    ("le", "≤"),
    ("ge", "≥"),
    ("ne", "≠"),
    ("equiv", "≡"),
    ("sim", "∼"),
    ("cong", "≅"),
    ("propto", "∝"),
    ("perp", "⊥"),
    ("partial", "∂"),
    ("nabla", "∇"),
    ("prod", "∏"),
    ("ast", "∗"),
    ("prime", "′"),
    ("cup", "∪"),
    ("cap", "∩"),
    ("subset", "⊂"),
    ("subseteq", "⊆"),
    ("supset", "⊃"),
    ("supseteq", "⊇"),
    ("notin", "∉"),
    ("ni", "∋"),
    ("emptyset", "∅"),
    ("varnothing", "∅"),
    ("oplus", "⊕"),
    ("otimes", "⊗"),
    ("wedge", "∧"),
    ("land", "∧"),
    ("lor", "∨"),
    ("to", "→"),
    ("rightarrow", "→"),
    ("leftarrow", "←"),
    ("gets", "←"),
    ("uparrow", "↑"),
    ("downarrow", "↓"),
    ("leftrightarrow", "↔"),
    // `\implies`/`\impliedby`/`\iff` are handled in `command_atom`: they
    // expand to a thick space, a long double arrow, and another thick space
    // (matching mathtools/plain TeX), not a bare glyph, so they are not rows
    // here.
    ("Leftarrow", "⇐"),
    ("Leftrightarrow", "⇔"),
    ("Uparrow", "⇑"),
    ("Downarrow", "⇓"),
    ("therefore", "∴"),
    ("angle", "∠"),
    ("aleph", "ℵ"),
    ("Re", "ℜ"),
    ("Im", "ℑ"),
    ("wp", "℘"),
    ("langle", "〈"),
    ("rangle", "〉"),
    ("lvert", "∣"),
    ("rvert", "∣"),
    // Symbol has no double bar U+2016: two real verticalbar glyphs.
    ("lVert", "∣∣"),
    ("rVert", "∣∣"),
    ("times", "×"),
    ("div", "÷"),
    ("pm", "±"),
    ("leq", "≤"),
    ("geq", "≥"),
    ("neq", "≠"),
    ("approx", "≈"),
    ("cdot", "⋅"),
    ("infty", "∞"),
    ("sum", "∑"),
    ("int", "∫"),
    ("in", "∈"),
    ("forall", "∀"),
    ("exists", "∃"),
    ("vee", "∨"),
    ("Rightarrow", "⇒"),
    ("mid", "∣"),
    // Neither glyph exists in Symbol.afm; both are drawn from the pinned
    // Latin Modern Math resource (`crate::lm_math`), not approximated.
    ("setminus", "∖"),
    ("Longrightarrow", "⟹"),
    // amssymb/latexsym symbols below have no base-14 Symbol glyph either;
    // all are drawn from the pinned Latin Modern Math resource (see issue #62).
    ("mp", "∓"),
    ("ll", "≪"),
    ("gg", "≫"),
    ("simeq", "≃"),
    ("vdots", "⋮"),
    ("ddots", "⋱"),
    ("lfloor", "⌊"),
    ("rfloor", "⌋"),
    ("lceil", "⌈"),
    ("rceil", "⌉"),
    ("oint", "∮"),
    ("mapsto", "↦"),
    ("ell", "ℓ"),
    ("hbar", "ℏ"),
    ("circ", "∘"),
    ("parallel", "∥"),
    ("nmid", "∤"),
    ("nleq", "≰"),
    ("ngeq", "≱"),
    ("subsetneq", "⊊"),
    ("supsetneq", "⊋"),
    ("lesssim", "≲"),
    ("gtrsim", "≳"),
    ("triangleq", "≜"),
    ("coloneqq", "≔"),
    ("nexists", "∄"),
    ("complement", "∁"),
    ("rightsquigarrow", "⇝"),
    ("hookrightarrow", "↪"),
    ("leftrightarrows", "⇆"),
    ("models", "⊨"),
    ("vdash", "⊢"),
    ("dashv", "⊣"),
    ("top", "⊤"),
    ("measuredangle", "∡"),
    ("square", "□"),
    ("blacksquare", "■"),
    ("lozenge", "◊"),
    ("checkmark", "✓"),
    // HW2 follow-up (issue #62): the remaining long arrows, drawn from the
    // pinned Latin Modern Math resource like `\Longrightarrow` above.
    ("Longleftrightarrow", "⟺"),
    ("longrightarrow", "⟶"),
    ("longleftarrow", "⟵"),
    ("Longleftarrow", "⟸"),
    ("longleftrightarrow", "⟷"),
    // `\triangle`, also from the pinned Latin Modern Math resource.
    // `\bigtriangleup` shares this exact glyph with a forced Bin class (see
    // `command_atom`), so it is not a second row here.
    ("triangle", "△"),
    ("bigtriangledown", "▽"),
    // `\bot` shares `\perp`'s exact base-14 Symbol glyph above with a forced
    // Ord class (see `command_atom`), so it is not a second row here.
];

/// Named operators typeset as upright roman words (`\sin x`, `\lim_{x\to 0}`).
const OPERATOR_NAMES: &[&str] = &[
    "sin", "cos", "tan", "cot", "sec", "csc", "arcsin", "arccos", "arctan", "sinh", "cosh", "tanh",
    "coth", "log", "ln", "lg", "exp", "lim", "liminf", "limsup", "max", "min", "sup", "inf", "det",
    "gcd", "deg", "dim", "ker", "arg", "hom", "Pr", "sgn",
];

/// Named commands that `\left`, `\right` and `\big...` accept as fences.
const DELIMITER_COMMANDS: &[&str] = &[
    "langle",
    "rangle",
    "lvert",
    "rvert",
    "lVert",
    "rVert",
    "lbrace",
    "rbrace",
    "uparrow",
    "downarrow",
    "Uparrow",
    "Downarrow",
    "lfloor",
    "rfloor",
    "lceil",
    "rceil",
];

fn text_atom(text: String, span: Span) -> MathAtom {
    MathAtom {
        nucleus: Nucleus::Text(text),
        span,
        superscript: None,
        subscript: None,
        class_override: None,
    }
}

/// The rule character used to draw fraction bars.
///
/// This is a stand-in, not a real glyph: runtime-v1 has no rule item type yet
/// (see issue #9). No font contains it, so it is deliberately unrepresentable in
/// the export adapter and is reported rather than silently substituted.
pub const FRACTION_RULE_CHAR: char = '\u{2500}';

/// The glyph a math-mode ASCII `-` renders as (U+2212, Symbol `minus`).
pub const MINUS_SIGN: &str = "\u{2212}";

fn command_glyph(name: &str) -> Option<&'static str> {
    COMMAND_GLYPHS
        .iter()
        .find(|(command, _)| *command == name)
        .map(|(_, glyph)| *glyph)
}

pub fn layout(list: &MathList, size: f64, diagnostics: &mut Vec<Diagnostic>) -> MathBox {
    layout_list(list, size, size, 0, diagnostics)
}

/// Display-style layout: scripts on `\lim`-like operators, `\sum` and `\prod`
/// at the top level stack centred above and below the operator, as in TeX.
pub fn layout_display(list: &MathList, size: f64, diagnostics: &mut Vec<Diagnostic>) -> MathBox {
    layout_list_with(list, size, size, 0, true, diagnostics)
}

/// Operators whose display-style scripts become limits.
fn takes_display_limits(nucleus: &Nucleus) -> bool {
    match nucleus {
        Nucleus::Text(name) => matches!(
            name.as_str(),
            "lim" | "liminf" | "limsup" | "max" | "min" | "sup" | "inf" | "det" | "gcd" | "Pr"
        ),
        Nucleus::Symbol(glyph) => matches!(glyph.as_str(), "∑" | "∏"),
        _ => false,
    }
}

/// TeX's atom classes (TeXbook Chapter 17), which drive inter-atom spacing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AtomClass {
    Ord,
    Op,
    Bin,
    Rel,
    Open,
    Close,
    Punct,
    Inner,
}

/// The class of `atom`, or `None` for explicit glue (`\,`, `\quad`, a null
/// `\left.`), which TeX skips when pairing atoms for spacing.
///
/// The parser does not keep TeX's class through `\left`/`\right` or
/// `\operatorname`, so a fence is classified by its glyph (open/close, not
/// inner) and `\operatorname{...}` text is ordinary. `atom.class_override`
/// (set by `\bot`, `\bigtriangleup`, and the `\mathbin`-family commands) wins
/// over any of that, including the explicit-glue case above, since an atom
/// with a forced class is never the invisible glue those commands produce.
fn atom_class(atom: &MathAtom) -> Option<AtomClass> {
    use AtomClass::*;
    if let Some(class) = atom.class_override {
        return Some(class);
    }
    Some(match &atom.nucleus {
        Nucleus::Space { .. } if atom.superscript.is_none() && atom.subscript.is_none() => {
            return None
        }
        Nucleus::Symbol(glyph) => symbol_class(glyph),
        Nucleus::Text(text) if OPERATOR_NAMES.contains(&text.as_str()) => Op,
        Nucleus::Text(text) if text == "mod" => Bin,
        Nucleus::Text(text) if text == "..." => Inner,
        Nucleus::Fraction { .. } => Inner,
        Nucleus::Matrix { left, right, .. } if !left.is_empty() || !right.is_empty() => Inner,
        // amsmath's `\overset`/`\stackrel` keep a relation or binary base's class.
        Nucleus::Stacked { base, .. } if base.atoms.len() == 1 => {
            match atom_class(&base.atoms[0]) {
                Some(class @ (Rel | Bin)) => class,
                _ => Ord,
            }
        }
        _ => Ord,
    })
}

fn symbol_class(glyph: &str) -> AtomClass {
    use AtomClass::*;
    match glyph {
        "=" | "<" | ">" | ":" | "≤" | "≥" | "≠" | "≈" | "≡" | "∼" | "≅" | "∝" | "⊥" | "∈" | "∉"
        | "∋" | "⊂" | "⊆" | "⊃" | "⊇" | "∣" | "→" | "←" | "↔" | "⇒" | "⇐" | "⇔" | "⟹" | "↑"
        | "↓" | "⇑" | "⇓" | "∴"
        // amssymb/latexsym relations, all drawn from the pinned Latin Modern
        // Math resource (`crate::lm_math`).
        | "≪" | "≫" | "≃" | "↦" | "∥" | "∤" | "≰" | "≱" | "⊊" | "⊋" | "≲" | "≳" | "≜" | "≔"
        | "⇝" | "↪" | "⇆" | "⊨" | "⊢" | "⊣"
        // HW2 follow-up: the remaining long arrows (issue #62), also from the
        // pinned Latin Modern Math resource. `⊥` above is `\perp`'s glyph;
        // `\bot` shares it but overrides the class to Ord (see `command_atom`).
        | "⟺" | "⟶" | "⟵" | "⟸" | "⟷" => Rel,
        "+" | "-" | "−" | "*" | "±" | "×" | "÷" | "⋅" | "·" | "∗" | "∪" | "∩" | "∨" | "∧" | "⊕"
        | "⊗" | "∖" | "∓" | "∘"
        // `\bigtriangledown`; `\bigtriangleup` shares `\triangle`'s glyph
        // (Ord by default here) and overrides its class to Bin instead.
        | "▽" => Bin,
        "(" | "[" | "{" | "〈" | "⟨" | "⌊" | "⌈" => Open,
        ")" | "]" | "}" | "〉" | "⟩" | "!" | "?" | "⌋" | "⌉" => Close,
        "," | ";" => Punct,
        "∑" | "∏" | "∫" | "∫∫" | "∫∫∫" | "∮" => Op,
        "⋅⋅⋅" => Inner,
        _ => Ord,
    }
}

/// Resolves each atom's class for spacing: TeX turns a binary operator with
/// no left operand (list start, or after Bin/Op/Rel/Open/Punct) into Ord, and
/// likewise one directly followed by Rel/Close/Punct or ending the list.
fn spacing_classes(list: &MathList) -> Vec<Option<AtomClass>> {
    use AtomClass::*;
    let mut classes: Vec<Option<AtomClass>> = list.atoms.iter().map(atom_class).collect();
    let mut previous: Option<usize> = None;
    for i in 0..classes.len() {
        let Some(class) = classes[i] else { continue };
        let before = previous.and_then(|p| classes[p]);
        match class {
            Bin if matches!(before, None | Some(Bin | Op | Rel | Open | Punct)) => {
                classes[i] = Some(Ord)
            }
            Rel | Close | Punct if before == Some(Bin) => classes[previous.unwrap()] = Some(Ord),
            _ => {}
        }
        previous = Some(i);
    }
    if let Some(last) = previous {
        if classes[last] == Some(Bin) {
            classes[last] = Some(Ord);
        }
    }
    classes
}

/// The TeXbook Chapter 18 spacing table, in mu (thin 3, medium 4, thick 5).
/// Entries TeX parenthesises apply only in display and text styles.
fn inter_atom_mu(left: AtomClass, right: AtomClass, script: bool) -> f64 {
    use AtomClass::*;
    let (mu, text_styles_only) = match (left, right) {
        (Ord | Close, Op) | (Op, Ord | Op) | (Inner, Op) => (3.0, false),
        (Ord | Op | Close | Inner, Bin) | (Bin, Ord | Op | Open | Inner) => (4.0, true),
        (Ord | Op | Close | Inner, Rel) | (Rel, Ord | Op | Open | Inner) => (5.0, true),
        (Ord | Op | Close, Inner) | (Inner, Ord | Open | Punct | Inner) => (3.0, true),
        (Punct, Ord | Op | Rel | Open | Close | Punct | Inner) => (3.0, true),
        _ => (0.0, false),
    };
    if script && text_styles_only {
        0.0
    } else {
        mu
    }
}

fn layout_list(
    list: &MathList,
    size: f64,
    root_size: f64,
    level: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> MathBox {
    layout_list_with(list, size, root_size, level, false, diagnostics)
}

fn layout_list_with(
    list: &MathList,
    size: f64,
    root_size: f64,
    level: usize,
    display: bool,
    diagnostics: &mut Vec<Diagnostic>,
) -> MathBox {
    let mut out = MathBox {
        items: Vec::new(),
        width: 0.0,
        ascent: size,
        descent: 0.2 * size,
    };
    let classes = spacing_classes(list);
    let mut previous_class = None;
    for (atom, class) in list.atoms.iter().zip(classes) {
        if let Some(class) = class {
            if let Some(previous) = previous_class {
                // Scripts and fraction parts are the only lists laid out
                // below level 0, so `level > 0` is TeX's script style.
                out.width += inter_atom_mu(previous, class, level > 0) / 18.0 * size;
            }
            previous_class = Some(class);
        }
        let mut nucleus = layout_nucleus(atom, size, root_size, level, diagnostics);
        if display
            && level == 0
            && (atom.superscript.is_some() || atom.subscript.is_some())
            && takes_display_limits(&atom.nucleus)
        {
            let script_size = root_size * SCRIPT_SCALE;
            let sup = atom
                .superscript
                .as_ref()
                .map(|l| layout_list(l, script_size, root_size, level + 1, diagnostics));
            let sub = atom
                .subscript
                .as_ref()
                .map(|l| layout_list(l, script_size, root_size, level + 1, diagnostics));
            let width = [Some(&nucleus), sup.as_ref(), sub.as_ref()]
                .into_iter()
                .flatten()
                .map(|b| b.width)
                .fold(0.0, f64::max);
            offset_items(
                &mut nucleus.items,
                out.width + (width - nucleus.width) / 2.0,
                0.0,
            );
            out.ascent = out.ascent.max(nucleus.ascent);
            out.descent = out.descent.max(nucleus.descent);
            out.items.extend(nucleus.items);
            // Limit baselines clear the operator's cap height / descender by
            // a small gap; box ascents include font-size headroom.
            if let Some(mut b) = sup {
                let dy = -0.8 * size - 0.12 * size - b.descent;
                offset_items(&mut b.items, out.width + (width - b.width) / 2.0, dy);
                out.ascent = out.ascent.max(b.ascent - dy);
                out.items.extend(b.items);
            }
            if let Some(mut b) = sub {
                let dy = 0.2 * size + 0.12 * size + 0.75 * script_size;
                offset_items(&mut b.items, out.width + (width - b.width) / 2.0, dy);
                out.descent = out.descent.max(b.descent + dy);
                out.items.extend(b.items);
            }
            out.width += width;
            continue;
        }
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
        // TeX's math `-` is the minus sign (Symbol `minus`), not a hyphen.
        Nucleus::Symbol(text) if text == "-" => layout_nucleus(
            &MathAtom {
                nucleus: Nucleus::Symbol(MINUS_SIGN.into()),
                span: atom.span,
                superscript: None,
                subscript: None,
                class_override: atom.class_override,
            },
            size,
            root_size,
            level,
            diagnostics,
        ),
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
            width: match (&atom.nucleus, crate::lm_math::width_pt(text, size)) {
                (Nucleus::Symbol(_), Some(width)) => width,
                _ => {
                    crate::layout::shaped_width(
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
                    .0
                }
            },
            ascent: size,
            descent: 0.2 * size,
        },
        Nucleus::Bold(text) => MathBox {
            items: vec![MathItem {
                font: Some(crate::layout::Font::TimesBold),
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
                crate::layout::Font::TimesBold,
                atom.span,
                diagnostics,
            )
            .0,
            ascent: size,
            descent: 0.2 * size,
        },
        Nucleus::Stacked { base, over, under } => {
            let script_size = if level == 0 {
                root_size * SCRIPT_SCALE
            } else {
                root_size * SECOND_ORDER_SCRIPT_SCALE
            };
            let mut b = layout_list(base, size, root_size, level, diagnostics);
            let over = over
                .as_ref()
                .map(|l| layout_list(l, script_size, root_size, level + 1, diagnostics));
            let under = under
                .as_ref()
                .map(|l| layout_list(l, script_size, root_size, level + 1, diagnostics));
            let width = [Some(&b), over.as_ref(), under.as_ref()]
                .into_iter()
                .flatten()
                .map(|m| m.width)
                .fold(0.0, f64::max);
            offset_items(&mut b.items, (width - b.width) / 2.0, 0.0);
            let mut out = MathBox {
                items: b.items,
                width,
                ascent: b.ascent,
                descent: b.descent,
            };
            if let Some(mut m) = over {
                let dy = -0.75 * size - 0.1 * size - m.descent;
                offset_items(&mut m.items, (width - m.width) / 2.0, dy);
                out.ascent = out.ascent.max(m.ascent - dy);
                out.items.extend(m.items);
            }
            if let Some(mut m) = under {
                let dy = 0.2 * size + 0.1 * size + 0.75 * script_size;
                offset_items(&mut m.items, (width - m.width) / 2.0, dy);
                out.descent = out.descent.max(m.descent + dy);
                out.items.extend(m.items);
            }
            out
        }
        Nucleus::Framed { body, frame } => {
            let mut b = layout_list(body, size, root_size, level, diagnostics);
            let rule = FRACTION_RULE_EM * size;
            let pad = if *frame == Frame::Box {
                0.25 * size
            } else {
                0.0
            };
            offset_items(&mut b.items, pad, 0.0);
            let width = b.width + 2.0 * pad;
            // Content extents: ascent/descent carry font-size headroom, so the
            // rules sit a small gap outside the nominal glyph box.
            let top = -(0.75 * size) - 0.15 * size - pad * 0.4;
            let bottom = 0.2 * size + 0.1 * size + pad * 0.4;
            let rule_item = |x: f64, y: f64, w: f64, h: f64| MathItem {
                font: None,
                text: FRACTION_RULE_CHAR.to_string(),
                x,
                baseline: y + h,
                size,
                span: atom.span,
                rule: Some(MathRule {
                    y,
                    width: w,
                    height: h,
                }),
            };
            let mut rules = Vec::new();
            if matches!(frame, Frame::Box | Frame::Over) {
                rules.push(rule_item(0.0, top - rule, width, rule));
            }
            if matches!(frame, Frame::Box | Frame::Under) {
                rules.push(rule_item(0.0, bottom, width, rule));
            }
            if *frame == Frame::Box {
                let height = bottom - top + 2.0 * rule;
                rules.push(rule_item(0.0, top - rule, rule, height));
                rules.push(rule_item(width - rule, top - rule, rule, height));
            }
            b.items.extend(rules);
            MathBox {
                items: b.items,
                width,
                ascent: b.ascent.max(-(top - rule)),
                descent: b.descent.max(bottom + rule),
            }
        }
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
            // The vinculum, as Symbol's own `radicalex` extender draws it: from
            // the radical's ink edge over the whole body, top-aligned with the
            // radical glyph. Neither the sign nor the bar grows for tall bodies.
            let vinculum_x = RADICAL_INK_RIGHT_EM * size;
            let vinculum_height = RADICALEX_THICKNESS_EM * size;
            let vinculum_y = -RADICAL_TOP_EM * size;
            b.items.push(MathItem {
                font: None,
                text: FRACTION_RULE_CHAR.to_string(),
                x: vinculum_x,
                baseline: vinculum_y + vinculum_height,
                size,
                span: atom.span,
                rule: Some(MathRule {
                    y: vinculum_y,
                    width: radical_width - vinculum_x + b.width,
                    height: vinculum_height,
                }),
            });
            b.width += radical_width;
            b.ascent = b.ascent.max(RADICAL_TOP_EM * size);
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
        Nucleus::Accent { accent, body } => {
            layout_accent(atom, *accent, body, size, root_size, level, diagnostics)
        }
        // `\mathbin{...}` and kin: laid out exactly like a bare `{...}`
        // group; only the enclosing atom's forced class differs.
        Nucleus::Group(body) => layout_list(body, size, root_size, level, diagnostics),
    }
}

/// Places `accent`'s mark over `body`.
///
/// Horizontal: plain symmetric centering, `(body.width - glyph.width) / 2`.
/// TeX adds a "skew" term here from the base character's TFM skewchar kern
/// (a slant correction for math-italic letters).
/// That term does not apply here: this compiler's math letters render in
/// upright Times-Roman, never math-italic (`crate::layout::math_font` never
/// selects an italic face), and Adobe Core 14 AFM metrics
/// (`crate::layout::x_height_pt`'s sibling, `Core14Face`) have no skewchar
/// kerning concept at all — that is a TeX TFM construct, not an AFM one.
/// Adding a foreign skew constant to an unslanted glyph would miscenter it,
/// not fix it.
///
/// Vertical: TeX's real rule, `raise = min(nucleus_height, accent font's
/// x-height)`, using the real Times-Roman x-height
/// (`crate::layout::x_height_pt`) rather than a guessed constant.
fn layout_accent(
    atom: &MathAtom,
    accent: Accent,
    body: &MathList,
    size: f64,
    root_size: f64,
    level: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> MathBox {
    let mut b = layout_list(body, size, root_size, level, diagnostics);
    let Some(glyph) = accent.glyph() else {
        // \check / \breve: no base-14 glyph. Already diagnosed at parse time
        // (`accent_atom`); typeset the body alone rather than draw nothing
        // and also fabricate a plausible-looking substitute mark.
        return b;
    };
    let glyph_text = glyph.to_string();
    let accent_font = crate::layout::math_font(&glyph_text);
    let accent_width =
        crate::layout::shaped_width(&glyph_text, size, accent_font, atom.span, diagnostics).0;
    let dx = (b.width - accent_width) / 2.0;
    let raise = b.ascent.min(crate::layout::x_height_pt(accent_font, size));
    // A thin mark, not a full-height glyph: ~0.15em is enough for a
    // circumflex/tilde/dot/acute stroke without inflating every accented
    // atom's box to a full line height.
    let accent_ascent = 0.15 * size;
    b.items.push(MathItem {
        font: Some(accent_font),
        text: glyph_text,
        x: dx,
        baseline: -raise,
        size,
        span: atom.span,
        rule: None,
    });
    b.ascent = b.ascent.max(raise + accent_ascent);
    b
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
            Nucleus::Bold(s) => Nucleus::Bold(s.clone()),
            Nucleus::Framed { body, frame } => Nucleus::Framed {
                body: shift_list(body, delta),
                frame: *frame,
            },
            Nucleus::Stacked { base, over, under } => Nucleus::Stacked {
                base: shift_list(base, delta),
                over: over.as_ref().map(|l| shift_list(l, delta)),
                under: under.as_ref().map(|l| shift_list(l, delta)),
            },
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
            Nucleus::Accent { accent, body } => Nucleus::Accent {
                accent: *accent,
                body: shift_list(body, delta),
            },
            Nucleus::Group(inner) => Nucleus::Group(shift_list(inner, delta)),
        },
        span: shift(atom.span, delta),
        superscript: atom.superscript.as_ref().map(|l| shift_list(l, delta)),
        subscript: atom.subscript.as_ref().map(|l| shift_list(l, delta)),
        class_override: atom.class_override,
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
    fn display_limits_stack_under_lim_but_stay_beside_inline_and_on_integrals() {
        let mut diagnostics = Vec::new();
        let tokens = crate::lexer::tokenize(r"\lim_{x\to 0} f \int_0^1 g");
        let list = parse_tokens(&tokens, &mut diagnostics);
        let x_of = |b: &MathBox, text: &str| {
            b.items
                .iter()
                .find(|item| item.text == text)
                .map(|item| (item.x, item.baseline))
                .unwrap()
        };
        let display = layout_display(&list, 12.0, &mut diagnostics);
        let inline = layout(&list, 12.0, &mut diagnostics);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        let (lim_x, _) = x_of(&display, "lim");
        let (sub_x, sub_y) = x_of(&display, "x");
        // Stacked: the limit starts under the operator, not after it.
        assert!(
            sub_x <= lim_x + 1.0 && sub_y > 0.5 * 12.0,
            "{lim_x} {sub_x} {sub_y}"
        );
        let (_, inline_sub_y) = x_of(&inline, "x");
        assert!(inline_sub_y < sub_y);
        // Integrals keep side scripts in display style.
        let (int_x, _) = x_of(&display, "∫");
        let zero_x = display
            .items
            .iter()
            .rev()
            .find(|item| item.text == "0")
            .unwrap()
            .x;
        assert!(zero_x > int_x);
        assert!(display.width < inline.width);
    }

    #[test]
    fn stacked_scripts_and_infix_choose_over_build_real_atoms() {
        let mut diagnostics = Vec::new();
        let tokens = crate::lexer::tokenize(
            r"\overset{?}{=} \underset{x}{\min} {n \choose k} {a \over b} \lim\limits_{x}",
        );
        let list = parse_tokens(&tokens, &mut diagnostics);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        let nuclei: Vec<&Nucleus> = list.atoms.iter().map(|atom| &atom.nucleus).collect();
        assert_eq!(nuclei.len(), 5, "{nuclei:?}");
        assert!(
            list.atoms[4].subscript.is_some(),
            "limits keeps the script on lim"
        );
        assert!(matches!(
            nuclei[0],
            Nucleus::Stacked {
                over: Some(_),
                under: None,
                ..
            }
        ));
        assert!(matches!(
            nuclei[1],
            Nucleus::Stacked {
                over: None,
                under: Some(_),
                ..
            }
        ));
        assert!(matches!(nuclei[2], Nucleus::Matrix { rows, .. } if rows.len() == 2));
        assert!(matches!(nuclei[3], Nucleus::Fraction { .. }));
        let laid = layout(&list, 12.0, &mut diagnostics);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        let question = laid.items.iter().find(|item| item.text == "?").unwrap();
        let equals = laid.items.iter().find(|item| item.text == "=").unwrap();
        assert!(question.baseline < equals.baseline - 6.0, "? sits above =");
    }

    #[test]
    fn structural_homework_commands_build_real_atoms() {
        let mut diagnostics = Vec::new();
        let tokens = crate::lexer::tokenize(
            r"\binom{n}{k} \sqrt[3]{8} \mathbf{F} \boxed{x=4} \overline{AB} a \pmod{n} \tag{2}",
        );
        let list = parse_tokens(&tokens, &mut diagnostics);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        let nuclei: Vec<&Nucleus> = list.atoms.iter().map(|atom| &atom.nucleus).collect();
        assert!(
            matches!(nuclei[0], Nucleus::Matrix { rows, left, .. } if rows.len() == 2 && left == "(")
        );
        assert!(
            list.atoms[1].superscript.is_some(),
            "root index is a raised script"
        );
        assert!(matches!(nuclei[2], Nucleus::Radical(_)));
        assert_eq!(nuclei[3], &Nucleus::Bold("F".into()));
        assert!(matches!(
            nuclei[4],
            Nucleus::Framed {
                frame: Frame::Box,
                ..
            }
        ));
        assert!(matches!(
            nuclei[5],
            Nucleus::Framed {
                frame: Frame::Over,
                ..
            }
        ));
        assert!(nuclei.contains(&&Nucleus::Text("(2)".into())));
        let laid = layout(&list, 12.0, &mut diagnostics);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        // The box contributes four real rules, the overline one and the
        // radical's vinculum one, beside the binomial's none.
        assert_eq!(
            laid.items.iter().filter(|item| item.rule.is_some()).count(),
            6
        );
    }

    #[test]
    fn sqrt_draws_its_vinculum_over_the_whole_body() {
        let mut diagnostics = Vec::new();
        let tokens = crate::lexer::tokenize(r"\sqrt{10-x}");
        let list = parse_tokens(&tokens, &mut diagnostics);
        let size = 10.0;
        let laid = layout(&list, size, &mut diagnostics);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        let rules: Vec<_> = laid.items.iter().filter(|i| i.rule.is_some()).collect();
        assert_eq!(rules.len(), 1, "exactly one vinculum");
        let bar = rules[0].rule.unwrap();
        let x_item = laid.items.iter().find(|i| i.text == "x").unwrap();
        // Starts at the radical's ink edge and reaches the end of the body.
        assert!((rules[0].x - RADICAL_INK_RIGHT_EM * size).abs() < 1e-9);
        assert!((rules[0].x + bar.width - laid.width).abs() < 1e-9);
        assert!(
            rules[0].x + bar.width > x_item.x,
            "covers the last body glyph"
        );
        // Top-aligned with the radical glyph at Symbol's radicalex thickness.
        assert!((bar.y + RADICAL_TOP_EM * size).abs() < 1e-9);
        assert!((bar.height - RADICALEX_THICKNESS_EM * size).abs() < 1e-9);
    }

    #[test]
    fn handwritten_homework_constructs_parse_and_shape_without_diagnostics() {
        let mut diagnostics = Vec::new();
        let tokens = crate::lexer::tokenize(
            r"\lim_{n\to\infty}\left(1+\frac{1}{n}\right)^n \sin\theta \operatorname*{rank}(A)
              \mathrm{d}x \Gamma\Delta\partial\nabla\equiv\propto\cup\subseteq\notin\emptyset
              \iff\langle u\rangle \big\{ \bigr\} \left. \right| \dfrac{1}{2} a\!b\cdots\dots",
        );
        let list = parse_tokens(&tokens, &mut diagnostics);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        let _ = layout(&list, 12.0, &mut diagnostics);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert!(list
            .atoms
            .iter()
            .any(|atom| atom.nucleus == Nucleus::Text("lim".into()) && atom.subscript.is_some()));
        assert!(list
            .atoms
            .iter()
            .any(|atom| atom.nucleus == Nucleus::Text("rank".into())));
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
mod accent_tests {
    use super::*;

    fn laid_out(source: &str, size: f64) -> (MathBox, Vec<Diagnostic>) {
        let mut diagnostics = Vec::new();
        let tokens = crate::lexer::tokenize(source);
        let list = parse_tokens(&tokens, &mut diagnostics);
        let b = layout(&list, size, &mut diagnostics);
        (b, diagnostics)
    }

    /// The measurement the task brief asked for: `\hat{A}`'s horizontal
    /// offset in this compiler. The math variable "A" is Times-Italic; the
    /// accent is still centred symmetrically, with no italic skew correction
    /// (a known limitation: TeX shifts accents right over slanted letters).
    #[test]
    fn hat_a_centers_symmetrically_with_no_skew_term() {
        let size = 10.0;
        let (b, diagnostics) = laid_out(r"\hat{A}", size);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        let a_item = b.items.iter().find(|i| i.text == "A").unwrap();
        let accent_item = b.items.iter().find(|i| i.text == "\u{2C6}").unwrap();

        let mut d = Vec::new();
        let a_width = crate::layout::shaped_width(
            "A",
            size,
            crate::layout::Font::TimesItalic,
            a_item.span,
            &mut d,
        )
        .0;
        let accent_width = crate::layout::shaped_width(
            "\u{2C6}",
            size,
            crate::layout::Font::TimesRoman,
            accent_item.span,
            &mut d,
        )
        .0;
        let expected_dx = (a_width - accent_width) / 2.0;

        assert!(
            (accent_item.x - a_item.x - expected_dx).abs() < 1e-9,
            "expected symmetric centering dx {expected_dx}, got {}",
            accent_item.x - a_item.x
        );
        // At 10pt Times-Italic A (611) and the circumflex (333) give ~1.39pt.
        assert!(
            (expected_dx - 1.39).abs() < 0.01,
            "expected ~1.39pt at 10pt, got {expected_dx}"
        );
    }

    #[test]
    fn accent_vertical_raise_is_capped_at_the_accent_fonts_x_height() {
        let size = 10.0;
        let (b, diagnostics) = laid_out(r"\hat{A}", size);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        let accent_item = b.items.iter().find(|i| i.text == "\u{2C6}").unwrap();
        let xheight = crate::layout::x_height_pt(crate::layout::Font::TimesRoman, size);
        assert!((accent_item.baseline - (-xheight)).abs() < 1e-9);
    }

    #[test]
    fn ddot_acute_grave_bar_use_exact_base14_glyphs() {
        for (source, glyph) in [
            (r"\ddot{x}", "\u{A8}"),
            (r"\acute{x}", "\u{B4}"),
            (r"\grave{x}", "\u{60}"),
            (r"\bar{x}", "\u{AF}"),
            (r"\tilde{x}", "\u{2DC}"),
        ] {
            let (b, diagnostics) = laid_out(source, 10.0);
            assert!(diagnostics.is_empty(), "{source}: {diagnostics:?}");
            assert!(
                b.items.iter().any(|i| i.text == glyph),
                "{source}: expected glyph {glyph:?} in {:?}",
                b.items.iter().map(|i| &i.text).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn check_and_breve_are_diagnosed_and_typeset_without_a_mark() {
        for source in [r"\check{x}", r"\breve{x}"] {
            let (b, diagnostics) = laid_out(source, 10.0);
            assert_eq!(diagnostics.len(), 1, "{source}: {diagnostics:?}");
            assert!(diagnostics[0]
                .message
                .contains("no representable accent glyph"));
            // Only the base "x", no extra accent glyph item.
            assert_eq!(b.items.len(), 1);
            assert_eq!(b.items[0].text, "x");
        }
    }

    #[test]
    fn widehat_over_one_symbol_is_silent_but_warns_over_more_than_one() {
        let (_, one) = laid_out(r"\widehat{A}", 10.0);
        assert!(one.is_empty(), "{one:?}");

        let (_, many) = laid_out(r"\widehat{AB}", 10.0);
        assert_eq!(many.len(), 1, "{many:?}");
        assert!(many[0].message.contains("does not stretch"));
    }

    #[test]
    fn overline_and_underline_draw_a_rule_spanning_the_body() {
        let size = 10.0;
        let (over, d1) = laid_out(r"\overline{x}", size);
        assert!(d1.is_empty(), "{d1:?}");
        let over_rule = over
            .items
            .iter()
            .find_map(|i| i.rule)
            .expect("overline rule");
        assert!(over_rule.width > 0.0);
        assert!(over_rule.height > 0.0);
        // Drawn above the body: strictly negative (upward) y.
        assert!(over_rule.y < 0.0);

        let (under, d2) = laid_out(r"\underline{x}", size);
        assert!(d2.is_empty(), "{d2:?}");
        let under_rule = under
            .items
            .iter()
            .find_map(|i| i.rule)
            .expect("underline rule");
        assert!(under_rule.width > 0.0);
        assert!(under_rule.height > 0.0);
        // Drawn below the body: strictly positive (downward) y.
        assert!(under_rule.y > 0.0);
    }
}

#[cfg(test)]
mod spacing_tests {
    use super::*;

    const SIZE: f64 = 18.0; // 1mu = 1pt

    fn laid_out(source: &str, size: f64) -> MathBox {
        let mut diagnostics = Vec::new();
        let list = parse_tokens(&crate::lexer::tokenize(source), &mut diagnostics);
        let b = layout(&list, size, &mut diagnostics);
        assert!(diagnostics.is_empty(), "{source}: {diagnostics:?}");
        b
    }

    fn width(source: &str, size: f64) -> f64 {
        laid_out(source, size).width
    }

    fn x(b: &MathBox, text: &str) -> f64 {
        b.items.iter().find(|i| i.text == text).unwrap().x
    }

    fn close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
    }

    #[test]
    fn relations_get_thick_space_on_both_sides() {
        let b = laid_out("a=b", SIZE);
        close(x(&b, "="), width("a", SIZE) + 5.0);
        close(x(&b, "b"), x(&b, "=") + width("=", SIZE) + 5.0);
    }

    #[test]
    fn binary_operators_get_medium_space() {
        let b = laid_out("a+b", SIZE);
        close(x(&b, "+"), width("a", SIZE) + 4.0);
        close(x(&b, "b"), x(&b, "+") + width("+", SIZE) + 4.0);
    }

    #[test]
    fn a_leading_or_post_relation_minus_is_ordinary_and_a_real_minus_sign() {
        let b = laid_out("-x", SIZE);
        assert!(b.items.iter().all(|i| i.text != "-"), "{:?}", b.items);
        close(x(&b, "x"), width("-", SIZE));
        close(b.width, width("-", SIZE) + width("x", SIZE));
        // After a relation: thick space before the minus, none after it.
        let b = laid_out("a=-x", SIZE);
        close(x(&b, MINUS_SIGN), x(&b, "=") + width("=", SIZE) + 5.0);
        close(x(&b, "x"), x(&b, MINUS_SIGN) + width("-", SIZE));
    }

    #[test]
    fn script_style_drops_relation_space() {
        let script = SIZE * SCRIPT_SCALE;
        let b = laid_out("a_{i=1}", SIZE);
        close(
            b.width,
            width("a", SIZE) + width("i", script) + width("=", script) + width("1", script),
        );
    }

    #[test]
    fn math_punctuation_gets_thin_space_after_only() {
        let b = laid_out("f(x),y", SIZE);
        close(x(&b, "("), width("f", SIZE));
        close(x(&b, "y"), x(&b, ",") + width(",", SIZE) + 3.0);
        close(b.width, width("f(x),y", SIZE));
        close(
            b.width,
            ["f", "(", "x", ")", ",", "y"]
                .iter()
                .map(|s| width(s, SIZE))
                .sum::<f64>()
                + 3.0,
        );
    }

    #[test]
    fn explicit_glue_adds_to_the_table_spacing() {
        close(width(r"a\,=b", SIZE), width("a=b", SIZE) + 3.0);
        close(
            width(r"\sin x", SIZE),
            width(r"\sin", SIZE) + width("x", SIZE) + 3.0,
        );
    }

    #[test]
    fn new_relations_get_thick_space_like_other_relations() {
        for command in [
            "ll",
            "gg",
            "simeq",
            "mapsto",
            "parallel",
            "nmid",
            "nleq",
            "ngeq",
            "subsetneq",
            "supsetneq",
            "lesssim",
            "gtrsim",
            "triangleq",
            "coloneqq",
            "rightsquigarrow",
            "hookrightarrow",
            "leftrightarrows",
            "models",
            "vdash",
            "dashv",
        ] {
            let glyph = command_glyph(command).unwrap();
            // A space after a control word is swallowed by the lexer (like
            // real TeX), so it safely separates the command from `b`.
            let b = laid_out(&format!(r"a\{command} b"), SIZE);
            close(x(&b, glyph), width("a", SIZE) + 5.0);
            close(x(&b, "b"), x(&b, glyph) + width(glyph, SIZE) + 5.0);
        }
    }

    #[test]
    fn mp_and_circ_get_medium_space_like_other_binary_operators() {
        for command in ["mp", "circ"] {
            let glyph = command_glyph(command).unwrap();
            let b = laid_out(&format!(r"a\{command} b"), SIZE);
            close(x(&b, glyph), width("a", SIZE) + 4.0);
            close(x(&b, "b"), x(&b, glyph) + width(glyph, SIZE) + 4.0);
        }
    }

    #[test]
    fn floor_and_ceiling_are_open_and_close_fences() {
        // Open fences get no leading space; close fences get no trailing space.
        let b = laid_out(r"a=\lfloor x\rfloor", SIZE);
        close(x(&b, "⌊"), x(&b, "=") + width("=", SIZE) + 5.0);
        close(x(&b, "x"), x(&b, "⌊") + width("⌊", SIZE));
        close(b.width, x(&b, "⌋") + width("⌋", SIZE));

        let b = laid_out(r"a=\lceil x\rceil", SIZE);
        close(x(&b, "⌈"), x(&b, "=") + width("=", SIZE) + 5.0);
        close(x(&b, "x"), x(&b, "⌈") + width("⌈", SIZE));
        close(b.width, x(&b, "⌉") + width("⌉", SIZE));
    }

    #[test]
    fn left_right_floor_and_ceiling_are_accepted_as_delimiters() {
        laid_out(r"\left\lfloor x \right\rfloor", SIZE);
        laid_out(r"\left\lceil x \right\rceil", SIZE);
    }

    #[test]
    fn oint_is_an_op_like_int_and_oint() {
        let b = laid_out(r"\oint_C f", SIZE);
        // Op class before an ordinary atom gets a thin space (3mu), same as \int.
        let int = laid_out(r"\int_C f", SIZE);
        close(b.width - width("∮", SIZE), int.width - width("∫", SIZE));
    }

    #[test]
    fn every_new_amssymb_command_renders_with_no_diagnostics() {
        for command in [
            "mp",
            "ll",
            "gg",
            "simeq",
            "vdots",
            "ddots",
            "lfloor",
            "rfloor",
            "lceil",
            "rceil",
            "oint",
            "mapsto",
            "ell",
            "hbar",
            "circ",
            "parallel",
            "nmid",
            "nleq",
            "ngeq",
            "subsetneq",
            "supsetneq",
            "lesssim",
            "gtrsim",
            "triangleq",
            "coloneqq",
            "nexists",
            "complement",
            "rightsquigarrow",
            "hookrightarrow",
            "leftrightarrows",
            "models",
            "vdash",
            "dashv",
            "top",
            "measuredangle",
            "square",
            "blacksquare",
            "lozenge",
            "checkmark",
        ] {
            laid_out(&format!(r"\{command}"), SIZE);
        }
    }

    /// Issue #62 HW2 follow-up: the remaining long arrows are Rel, same as
    /// the existing short arrows and `\Longrightarrow`.
    #[test]
    fn long_arrows_get_thick_space_like_other_relations() {
        for command in [
            "Longleftrightarrow",
            "longrightarrow",
            "longleftarrow",
            "Longleftarrow",
            "longleftrightarrow",
        ] {
            let glyph = command_glyph(command).unwrap();
            let b = laid_out(&format!(r"a\{command} b"), SIZE);
            close(x(&b, glyph), width("a", SIZE) + 5.0);
            close(x(&b, "b"), x(&b, glyph) + width(glyph, SIZE) + 5.0);
        }
    }

    /// `\iff`/`\implies`/`\impliedby` expand to a thick space, the long
    /// double arrow, and another thick space (mathtools/plain TeX) — deliberate
    /// extra room on top of the automatic Rel spacing the arrow already gets,
    /// exactly like plain TeX's real `\def\iff{\;\Longleftrightarrow\;}`.
    #[test]
    fn iff_implies_impliedby_expand_to_a_spaced_long_arrow() {
        for (command, arrow) in [("iff", "⟺"), ("implies", "⟹"), ("impliedby", "⟸")] {
            let b = laid_out(&format!(r"a\{command} b"), SIZE);
            close(x(&b, arrow), width("a", SIZE) + 10.0);
            close(x(&b, "b"), x(&b, arrow) + width(arrow, SIZE) + 10.0);
        }
    }

    /// `\triangle` is Ord (no space against an adjacent ordinary atom);
    /// `\bigtriangleup` renders the identical glyph but is Bin.
    #[test]
    fn triangle_is_ord_and_bigtriangleup_is_bin_on_the_same_glyph() {
        let ord = laid_out(r"a\triangle b", SIZE);
        close(x(&ord, "△"), width("a", SIZE));
        close(x(&ord, "b"), x(&ord, "△") + width("△", SIZE));

        let bin = laid_out(r"a\bigtriangleup b", SIZE);
        close(x(&bin, "△"), width("a", SIZE) + 4.0);
        close(x(&bin, "b"), x(&bin, "△") + width("△", SIZE) + 4.0);
    }

    #[test]
    fn bigtriangledown_is_a_distinct_bin_glyph() {
        let b = laid_out(r"a\bigtriangledown b", SIZE);
        close(x(&b, "▽"), width("a", SIZE) + 4.0);
        close(x(&b, "b"), x(&b, "▽") + width("▽", SIZE) + 4.0);
    }

    /// `\bot` and `\perp` render the exact same U+22A5 glyph but must space
    /// differently: `\bot` is Ord (no relation space), `\perp` is Rel (thick
    /// space on both sides).
    #[test]
    fn bot_and_perp_render_the_same_glyph_with_different_spacing() {
        let bot = laid_out(r"a\bot b", SIZE);
        close(x(&bot, "⊥"), width("a", SIZE));
        close(x(&bot, "b"), x(&bot, "⊥") + width("⊥", SIZE));

        let perp = laid_out(r"a\perp b", SIZE);
        close(x(&perp, "⊥"), width("a", SIZE) + 5.0);
        close(x(&perp, "b"), x(&perp, "⊥") + width("⊥", SIZE) + 5.0);

        assert!(bot.width < perp.width);
    }

    /// TeXbook Chapter 17's `\mathbin`/`\mathrel`/`\mathord`/`\mathop`/
    /// `\mathopen`/`\mathclose`/`\mathpunct`: the class is forced regardless
    /// of what the argument's own atoms would otherwise imply.
    #[test]
    fn math_class_family_forces_spacing_around_an_arbitrary_argument() {
        // HW2 uses `\mathbin{\triangle}` for symmetric difference: it must
        // get Bin (medium) spacing, unlike bare `\triangle` above.
        let b = laid_out(r"A\mathbin{\triangle}B", SIZE);
        close(x(&b, "△"), width("A", SIZE) + 4.0);
        close(x(&b, "B"), x(&b, "△") + width("△", SIZE) + 4.0);

        // `\mathord{=}` strips the relation spacing a bare `=` would get.
        let ord = laid_out(r"a\mathord{=}b", SIZE);
        close(x(&ord, "="), width("a", SIZE));
        close(x(&ord, "b"), x(&ord, "=") + width("=", SIZE));

        // `\mathrel{+}` adds relation (thick) spacing a bare `+` would not get.
        let rel = laid_out(r"a\mathrel{+}b", SIZE);
        close(x(&rel, "+"), width("a", SIZE) + 5.0);
        close(x(&rel, "b"), x(&rel, "+") + width("+", SIZE) + 5.0);

        // A multi-atom argument is boxed as one atom: `\mathbin{ab}` spaces
        // like a single Bin atom around the whole two-letter group, not like
        // two separate ordinary atoms with no internal gap removed.
        let group = laid_out(r"A\mathbin{ab}B", SIZE);
        close(x(&group, "a"), width("A", SIZE) + 4.0);
        close(x(&group, "b"), x(&group, "a") + width("a", SIZE));
        close(x(&group, "B"), x(&group, "b") + width("b", SIZE) + 4.0);
    }

    #[test]
    fn qed_glyph_is_in_the_pinned_font_and_carries_no_export_loss() {
        assert!(crate::lm_math::advance('\u{220E}').is_some());
        assert!(matches!(
            crate::export::map_char('\u{220E}'),
            crate::export::Glyph::LatinModernMath
        ));
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
                        Nucleus::Bold(_) => usize::MAX,
                        Nucleus::Framed { body, .. } => min_start(body),
                        Nucleus::Stacked { base, over, under } => {
                            [Some(base), over.as_ref(), under.as_ref()]
                                .into_iter()
                                .flatten()
                                .map(min_start)
                                .min()
                                .unwrap_or(usize::MAX)
                        }
                        Nucleus::Matrix { rows, .. } => rows
                            .iter()
                            .flatten()
                            .map(min_start)
                            .min()
                            .unwrap_or(usize::MAX),
                        Nucleus::Accent { body, .. } => min_start(body),
                        Nucleus::Group(body) => min_start(body),
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
