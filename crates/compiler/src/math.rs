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
    Fraction {
        numerator: MathList,
        denominator: MathList,
    },
    Radical(MathList),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MathItem {
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
    let last_span = split.last().map(|t| t.span);
    let mut parser = MathParser {
        tokens: &split,
        i: 0,
        depth: 0,
        delimiter_depth: 0,
        diagnostics,
    };
    let list = parser.list(false);
    // A \left with no \right is an error in TeX, and silently accepting it would
    // let a half-typed formula look finished. Reported here, at the end of the
    // formula, because that is the first point at which it is known.
    if parser.delimiter_depth > 0 {
        let open = parser.delimiter_depth;
        parser.diagnostics.push(Diagnostic::error(
            format!("{open} \\left delimiter(s) without a matching \\right"),
            last_span,
            Some("typeset the opening delimiter and continued".into()),
        ));
    }
    list
}

/// Maximum nesting of braced math groups, scripts, fractions and radicals.
///
/// The list parser is recursive descent, so a document full of unclosed openers
/// recurses once per opener. Without a bound, a pathological file — or a
/// half-typed one — overflows the stack and kills the worker mid-keystroke.
/// Exceeding the bound is an explicit diagnostic, not a crash.
pub const MAX_MATH_DEPTH: usize = 256;

/// Maximum nesting of \left ... \right pairs.
pub const MAX_DELIMITER_DEPTH: usize = 64;

struct MathParser<'a> {
    tokens: &'a [Token],
    i: usize,
    depth: usize,
    /// Open \left delimiters, so an unmatched \right is diagnosed rather than
    /// silently accepted.
    delimiter_depth: usize,
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

    /// Consumes the delimiter after \left or \right.
    ///
    /// A full stop is TeX's null delimiter: it pairs but renders nothing.
    fn take_delimiter(&mut self) -> Option<(String, Span)> {
        while matches!(
            self.tokens.get(self.i).map(|t| &t.kind),
            Some(TokenKind::Space)
        ) {
            self.i += 1;
        }
        let token = self.tokens.get(self.i)?.clone();
        let text = match &token.kind {
            TokenKind::Word(w) if w == "." => String::new(),
            TokenKind::Word(w) => w.clone(),
            TokenKind::LBrace => "{".into(),
            TokenKind::RBrace => "}".into(),
            TokenKind::Command(c) => command_glyph(c).unwrap_or("").to_string(),
            _ => return None,
        };
        self.i += 1;
        Some((text, token.span))
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
            "sqrt" => MathAtom {
                nucleus: Nucleus::Radical(self.required_group("sqrt", span)),
                span,
                superscript: None,
                subscript: None,
            },
            // \left and \right delimit a subformula. The delimiter that follows
            // is emitted as an ordinary symbol: this removes the blocker and the
            // leaked literal text, but the delimiter is NOT grown to the height
            // of its content, which real TeX does by assembling extensible
            // pieces. That limitation is stated in README.md rather than implied.
            "left" | "right" => {
                let is_left = name == "left";
                if is_left {
                    self.delimiter_depth += 1;
                    if self.delimiter_depth > MAX_DELIMITER_DEPTH {
                        self.diagnostics.push(Diagnostic::error(
                            format!(
                                "\\left nesting deeper than {MAX_DELIMITER_DEPTH} levels is not supported"
                            ),
                            Some(span),
                            Some("stopped tracking delimiter pairing at this depth".into()),
                        ));
                    }
                } else if self.delimiter_depth == 0 {
                    self.diagnostics.push(Diagnostic::error(
                        "\\right has no matching \\left".to_string(),
                        Some(span),
                        Some("typeset the delimiter on its own and continued".into()),
                    ));
                } else {
                    self.delimiter_depth -= 1;
                }
                match self.take_delimiter() {
                    Some((text, delim_span)) => symbol(text, delim_span),
                    None => {
                        self.diagnostics.push(Diagnostic::error(
                            format!("\\{name} must be followed by a delimiter"),
                            Some(span),
                            Some("used no delimiter and continued".into()),
                        ));
                        symbol(String::new(), span)
                    }
                }
            }
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
    ("nu", "ν"),
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
    ("Gamma", "\u{393}"),
    ("Delta", "\u{394}"),
    ("Theta", "\u{398}"),
    ("Lambda", "\u{39B}"),
    ("Xi", "\u{39E}"),
    ("Pi", "\u{3A0}"),
    ("Sigma", "\u{3A3}"),
    ("Upsilon", "\u{3A5}"),
    ("Phi", "\u{3A6}"),
    ("Psi", "\u{3A8}"),
    ("Omega", "\u{3A9}"),
    ("partial", "\u{2202}"),
    ("nabla", "\u{2207}"),
    ("in", "\u{2208}"),
    ("prod", "\u{220F}"),
    ("to", "\u{2192}"),
    ("gets", "\u{2190}"),
    ("Rightarrow", "\u{21D2}"),
    ("Leftrightarrow", "\u{21D4}"),
    ("wedge", "\u{2227}"),
    ("vee", "\u{2228}"),
    ("neg", "\u{AC}"),
    ("forall", "\u{2200}"),
    ("exists", "\u{2203}"),
    ("emptyset", "\u{2205}"),
    ("equiv", "\u{2261}"),
    ("sim", "\u{223C}"),
    ("subset", "\u{2282}"),
    ("subseteq", "\u{2286}"),
    ("perp", "\u{22A5}"),
    ("angle", "\u{2220}"),
    ("ni", "\u{220B}"),
    ("notin", "\u{2209}"),
    ("supset", "\u{2283}"),
    ("supseteq", "\u{2287}"),
    ("cup", "\u{222A}"),
    ("cap", "\u{2229}"),
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
        Nucleus::Symbol(text) => MathBox {
            items: vec![MathItem {
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
                crate::layout::math_font(text),
                atom.span,
                diagnostics,
            )
            .0,
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
            Nucleus::Fraction {
                numerator,
                denominator,
            } => Nucleus::Fraction {
                numerator: shift_list(numerator, delta),
                denominator: shift_list(denominator, delta),
            },
            Nucleus::Radical(inner) => Nucleus::Radical(shift_list(inner, delta)),
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
                        Nucleus::Fraction {
                            numerator,
                            denominator,
                        } => min_start(numerator).min(min_start(denominator)),
                        Nucleus::Radical(inner) => min_start(inner),
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
