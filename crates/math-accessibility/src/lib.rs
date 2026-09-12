//! An original semantic accessibility adapter over `flashtex-math-layout`'s
//! math-list model ([`MathList`]/[`Atom`]/[`Nucleus`]).
//!
//! [`MathAccessibility::describe`] walks the *semantic* tree (not the laid-out
//! box tree) and produces:
//!
//! - `readable`: a structured, screen-reader-style spoken rendering.
//! - `mathml`: a standalone `<math>` document, correctly XML-escaped, that
//!   always preserves the exact source glyph identity of every symbol.
//! - `unsupported`: an explicit, typed list of every node this adapter could
//!   not give a semantic name to. Nothing is silently skipped, and nothing is
//!   guessed: an unnamed symbol never gets a plausible-looking spoken word it
//!   was not actually given.
//!
//! Recursion over nested math (groups, fractions, radicals, accents,
//! delimited bodies, sub/superscripts) is bounded by [`MathAccessibility::with_max_depth`]
//! ([`DEFAULT_MAX_DEPTH`] by default); exceeding it is a typed
//! [`AccessibilityError`], never a stack overflow.

use flashtex_math_layout::{Atom, MathList, Nucleus};

/// Default bound on math-list nesting depth (see [`MathAccessibility::with_max_depth`]).
pub const DEFAULT_MAX_DEPTH: usize = 64;

/// One step of the path from the root [`MathList`] to a node, used to locate
/// entries in [`Description::unsupported`] and the site of a
/// [`AccessibilityError::RecursionLimitExceeded`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathStep {
    /// The `n`th atom of the enclosing list.
    Atom(usize),
    /// Into a braced group (`Nucleus::List`).
    Group,
    /// Into an atom's superscript.
    Superscript,
    /// Into an atom's subscript.
    Subscript,
    /// Into a fraction's numerator.
    Numerator,
    /// Into a fraction's denominator.
    Denominator,
    /// Into a radical's radicand.
    Radicand,
    /// Into an accent's base.
    AccentBase,
    /// The accent glyph itself (not the base).
    AccentGlyph,
    /// Into a `\left ... \right` body.
    DelimitedBody,
    /// The left delimiter glyph.
    LeftDelimiter,
    /// The right delimiter glyph.
    RightDelimiter,
}

/// A path from the root list to a specific node.
pub type NodePath = Vec<PathStep>;

/// Why a node could not be given a semantic reading. The character itself is
/// always kept, so the caller can see exactly what was not understood.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnsupportedReason {
    /// This symbol has no entry in the spoken-name table.
    UnknownSymbolName(char),
    /// This accent character has no entry in the accent-name table.
    UnknownAccentName(char),
    /// The character is a control character and cannot be a math symbol or
    /// accent; it also cannot be embedded literally in MathML text.
    ControlCharacter(char),
}

/// One node this adapter explicitly declined to approximate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unsupported {
    pub path: NodePath,
    pub reason: UnsupportedReason,
}

/// A typed failure. Recursion is the only fallible condition: everything else
/// degrades to an explicit [`Unsupported`] marker instead of failing outright.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessibilityError {
    /// Math-list nesting exceeded the configured bound.
    RecursionLimitExceeded { max_depth: usize, path: NodePath },
}

impl std::fmt::Display for AccessibilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessibilityError::RecursionLimitExceeded { max_depth, path } => write!(
                f,
                "math list nesting exceeded the bound of {max_depth} level(s) at {path:?}"
            ),
        }
    }
}

impl std::error::Error for AccessibilityError {}

/// The result of describing a [`MathList`].
#[derive(Debug, Clone, PartialEq)]
pub struct Description {
    /// A structured, spoken-word rendering of the whole list.
    pub readable: String,
    /// A standalone, XML-escaped `<math>...</math>` document.
    pub mathml: String,
    /// Every node that could not be given a semantic name, in traversal order.
    pub unsupported: Vec<Unsupported>,
}

impl Description {
    /// `true` if every node in the tree was understood.
    pub fn is_fully_supported(&self) -> bool {
        self.unsupported.is_empty()
    }
}

/// A semantic math-accessibility adapter, configured with a recursion bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MathAccessibility {
    max_depth: usize,
}

impl Default for MathAccessibility {
    fn default() -> Self {
        Self::new()
    }
}

impl MathAccessibility {
    /// A new adapter with [`DEFAULT_MAX_DEPTH`] as its nesting bound.
    pub fn new() -> Self {
        MathAccessibility {
            max_depth: DEFAULT_MAX_DEPTH,
        }
    }

    /// A new adapter with an explicit nesting bound.
    pub fn with_max_depth(max_depth: usize) -> Self {
        MathAccessibility { max_depth }
    }

    /// Describes `list`, returning structured readable text and MathML, or a
    /// typed error if the list nests deeper than this adapter's bound.
    pub fn describe(&self, list: &MathList) -> Result<Description, AccessibilityError> {
        let mut renderer = Renderer {
            max_depth: self.max_depth,
            unsupported: Vec::new(),
        };
        let mut path = Vec::new();
        let rendered = renderer.render_list(list, 0, &mut path)?;
        Ok(Description {
            readable: rendered.readable,
            mathml: format!(
                "<math xmlns=\"http://www.w3.org/1998/Math/MathML\">{}</math>",
                rendered.mathml
            ),
            unsupported: renderer.unsupported,
        })
    }
}

/// The readable text and MathML fragment produced for one node.
struct Rendered {
    readable: String,
    mathml: String,
}

struct Renderer {
    max_depth: usize,
    unsupported: Vec<Unsupported>,
}

impl Renderer {
    fn check_depth(&self, depth: usize, path: &[PathStep]) -> Result<(), AccessibilityError> {
        if depth > self.max_depth {
            Err(AccessibilityError::RecursionLimitExceeded {
                max_depth: self.max_depth,
                path: path.to_vec(),
            })
        } else {
            Ok(())
        }
    }

    fn render_list(
        &mut self,
        list: &MathList,
        depth: usize,
        path: &mut Vec<PathStep>,
    ) -> Result<Rendered, AccessibilityError> {
        self.check_depth(depth, path)?;
        if list.atoms.is_empty() {
            return Ok(Rendered {
                readable: String::new(),
                mathml: "<mrow></mrow>".to_string(),
            });
        }
        let mut readable_parts = Vec::with_capacity(list.atoms.len());
        let mut mathml_parts = Vec::with_capacity(list.atoms.len());
        for (i, atom) in list.atoms.iter().enumerate() {
            path.push(PathStep::Atom(i));
            let rendered = self.render_atom(atom, depth, path)?;
            path.pop();
            if !rendered.readable.is_empty() {
                readable_parts.push(rendered.readable);
            }
            mathml_parts.push(rendered.mathml);
        }
        Ok(Rendered {
            readable: readable_parts.join(" "),
            mathml: format!("<mrow>{}</mrow>", mathml_parts.concat()),
        })
    }

    fn render_atom(
        &mut self,
        atom: &Atom,
        depth: usize,
        path: &mut Vec<PathStep>,
    ) -> Result<Rendered, AccessibilityError> {
        let nucleus = self.render_nucleus(&atom.nucleus, depth, path)?;
        match (&atom.superscript, &atom.subscript) {
            (None, None) => Ok(nucleus),
            (Some(sup), None) => {
                path.push(PathStep::Superscript);
                let sup = self.render_list(sup, depth + 1, path)?;
                path.pop();
                Ok(Rendered {
                    readable: format!("{} superscript {}", nucleus.readable, sup.readable),
                    mathml: format!("<msup>{}{}</msup>", nucleus.mathml, sup.mathml),
                })
            }
            (None, Some(sub)) => {
                path.push(PathStep::Subscript);
                let sub = self.render_list(sub, depth + 1, path)?;
                path.pop();
                Ok(Rendered {
                    readable: format!("{} subscript {}", nucleus.readable, sub.readable),
                    mathml: format!("<msub>{}{}</msub>", nucleus.mathml, sub.mathml),
                })
            }
            (Some(sup), Some(sub)) => {
                path.push(PathStep::Subscript);
                let sub = self.render_list(sub, depth + 1, path)?;
                path.pop();
                path.push(PathStep::Superscript);
                let sup = self.render_list(sup, depth + 1, path)?;
                path.pop();
                Ok(Rendered {
                    readable: format!(
                        "{} subscript {} superscript {}",
                        nucleus.readable, sub.readable, sup.readable
                    ),
                    // MathML requires msubsup's children in (base, sub, sup) order.
                    mathml: format!(
                        "<msubsup>{}{}{}</msubsup>",
                        nucleus.mathml, sub.mathml, sup.mathml
                    ),
                })
            }
        }
    }

    fn render_nucleus(
        &mut self,
        nucleus: &Nucleus,
        depth: usize,
        path: &mut Vec<PathStep>,
    ) -> Result<Rendered, AccessibilityError> {
        match nucleus {
            Nucleus::Symbol(ch) => Ok(self.render_symbol(*ch, path)),
            Nucleus::List(list) => {
                path.push(PathStep::Group);
                let rendered = self.render_list(list, depth + 1, path)?;
                path.pop();
                Ok(rendered)
            }
            Nucleus::Fraction {
                numerator,
                denominator,
                thickness,
            } => {
                path.push(PathStep::Numerator);
                let num = self.render_list(numerator, depth + 1, path)?;
                path.pop();
                path.push(PathStep::Denominator);
                let den = self.render_list(denominator, depth + 1, path)?;
                path.pop();
                let mathml = match thickness {
                    // The exact rule thickness is preserved verbatim rather than
                    // collapsed into a "thin"/"thick" approximation.
                    Some(t) => format!(
                        "<mfrac linethickness=\"{t}pt\">{}{}</mfrac>",
                        num.mathml, den.mathml
                    ),
                    None => format!("<mfrac>{}{}</mfrac>", num.mathml, den.mathml),
                };
                Ok(Rendered {
                    readable: format!(
                        "start fraction, {}, over, {}, end fraction",
                        num.readable, den.readable
                    ),
                    mathml,
                })
            }
            Nucleus::Radical(radicand) => {
                path.push(PathStep::Radicand);
                let radicand = self.render_list(radicand, depth + 1, path)?;
                path.pop();
                Ok(Rendered {
                    readable: format!(
                        "start square root of {}, end square root",
                        radicand.readable
                    ),
                    mathml: format!("<msqrt>{}</msqrt>", radicand.mathml),
                })
            }
            Nucleus::Accent { accent, base } => {
                path.push(PathStep::AccentBase);
                let base = self.render_list(base, depth + 1, path)?;
                path.pop();
                path.push(PathStep::AccentGlyph);
                let (accent_word, accent_mathml) = self.render_accent_glyph(*accent, path);
                path.pop();
                Ok(Rendered {
                    readable: format!("{} with {} accent", base.readable, accent_word),
                    mathml: format!(
                        "<mover accent=\"true\">{}{}</mover>",
                        base.mathml, accent_mathml
                    ),
                })
            }
            Nucleus::Delimited { left, right, body } => {
                let left = self.render_delimiter(*left, PathStep::LeftDelimiter, path);
                path.push(PathStep::DelimitedBody);
                let body = self.render_list(body, depth + 1, path)?;
                path.pop();
                let right = self.render_delimiter(*right, PathStep::RightDelimiter, path);
                let readable = [left.readable, body.readable, right.readable]
                    .into_iter()
                    .filter(|part| !part.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ");
                Ok(Rendered {
                    readable,
                    mathml: format!(
                        "<mrow>{}{}{}</mrow>",
                        left.mathml, body.mathml, right.mathml
                    ),
                })
            }
            Nucleus::Empty => Ok(Rendered {
                readable: String::new(),
                mathml: "<mrow></mrow>".to_string(),
            }),
        }
    }

    /// Renders a single symbol character. Never guesses: a character with no
    /// spoken-name entry becomes an explicit `[unsupported symbol U+XXXX]`
    /// marker recorded in `self.unsupported`, and control characters (which
    /// cannot legally appear as literal MathML text) become an explicit
    /// `<merror>` marker instead of a raw byte.
    fn render_symbol(&mut self, ch: char, path: &[PathStep]) -> Rendered {
        if ch.is_control() {
            self.unsupported.push(Unsupported {
                path: path.to_vec(),
                reason: UnsupportedReason::ControlCharacter(ch),
            });
            let note = format!("unsupported control character U+{:04X}", ch as u32);
            return Rendered {
                readable: format!("[{note}]"),
                mathml: format!("<merror><mtext>{note}</mtext></merror>"),
            };
        }
        let tag = mathml_tag_for_symbol(ch);
        let mathml = format!("<{tag}>{}</{tag}>", escape_xml_text(&ch.to_string()));
        let readable = if ch.is_ascii_alphanumeric() {
            ch.to_string()
        } else if let Some(name) = named_symbol(ch) {
            name.to_string()
        } else {
            self.unsupported.push(Unsupported {
                path: path.to_vec(),
                reason: UnsupportedReason::UnknownSymbolName(ch),
            });
            format!("[unsupported symbol U+{:04X}]", ch as u32)
        };
        Rendered { readable, mathml }
    }

    fn render_accent_glyph(&mut self, accent: char, path: &[PathStep]) -> (String, String) {
        if accent.is_control() {
            self.unsupported.push(Unsupported {
                path: path.to_vec(),
                reason: UnsupportedReason::ControlCharacter(accent),
            });
            let note = format!("unsupported control character U+{:04X}", accent as u32);
            return (
                format!("[{note}]"),
                format!("<merror><mtext>{note}</mtext></merror>"),
            );
        }
        // The exact accent glyph is always preserved in MathML, whether or not
        // we can name it for speech.
        let mathml = format!("<mo>{}</mo>", escape_xml_text(&accent.to_string()));
        match accent_name(accent) {
            Some(name) => (name.to_string(), mathml),
            None => {
                self.unsupported.push(Unsupported {
                    path: path.to_vec(),
                    reason: UnsupportedReason::UnknownAccentName(accent),
                });
                (
                    format!("[unsupported accent U+{:04X}]", accent as u32),
                    mathml,
                )
            }
        }
    }

    fn render_delimiter(
        &mut self,
        ch: Option<char>,
        step: PathStep,
        path: &mut Vec<PathStep>,
    ) -> Rendered {
        match ch {
            None => Rendered {
                readable: String::new(),
                mathml: String::new(),
            },
            Some(ch) => {
                path.push(step);
                let rendered = self.render_symbol(ch, path);
                path.pop();
                rendered
            }
        }
    }
}

fn mathml_tag_for_symbol(ch: char) -> &'static str {
    if ch.is_ascii_digit() {
        "mn"
    } else if ch.is_alphabetic() {
        "mi"
    } else {
        "mo"
    }
}

/// Escapes text for use inside MathML element content. Non-ASCII characters
/// are left as literal UTF-8, which is valid XML content and preserves the
/// exact source glyph.
fn escape_xml_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

/// Spoken names for symbols with no obvious single-glyph reading. ASCII
/// letters and digits are spoken as themselves and never consult this table.
fn named_symbol(ch: char) -> Option<&'static str> {
    Some(match ch {
        '+' => "plus",
        '-' | '\u{2212}' => "minus",
        '*' => "times",
        '/' => "divided by",
        '=' => "equals",
        '<' => "less than",
        '>' => "greater than",
        ':' => "colon",
        ',' => "comma",
        ';' => "semicolon",
        '.' => "dot",
        '!' => "factorial",
        '(' => "open parenthesis",
        ')' => "close parenthesis",
        '[' => "open bracket",
        ']' => "close bracket",
        '{' => "open brace",
        '}' => "close brace",
        '|' => "vertical bar",
        '\u{27E8}' => "open angle bracket",
        '\u{27E9}' => "close angle bracket",
        '\u{2308}' => "left ceiling",
        '\u{2309}' => "right ceiling",
        '\u{230A}' => "left floor",
        '\u{230B}' => "right floor",
        '\u{00D7}' => "times",
        '\u{00F7}' => "divided by",
        '\u{00B1}' => "plus minus",
        '\u{2213}' => "minus plus",
        '\u{2217}' => "asterisk",
        '\u{2218}' => "composed with",
        '\u{2229}' => "intersect",
        '\u{222A}' => "union",
        '\u{2228}' => "or",
        '\u{2227}' => "and",
        '\u{2295}' => "circled plus",
        '\u{2297}' => "circled times",
        '\u{2216}' => "set minus",
        '\u{2264}' => "less than or equal to",
        '\u{2265}' => "greater than or equal to",
        '\u{2261}' => "equivalent to",
        '\u{2248}' => "approximately equal to",
        '\u{2260}' => "not equal to",
        '\u{223C}' => "similar to",
        '\u{2282}' => "subset of",
        '\u{2283}' => "superset of",
        '\u{2286}' => "subset of or equal to",
        '\u{2287}' => "superset of or equal to",
        '\u{2208}' => "element of",
        '\u{220B}' => "contains",
        '\u{2190}' => "left arrow",
        '\u{2192}' => "right arrow",
        '\u{2194}' => "left right arrow",
        '\u{21D0}' => "implied by",
        '\u{21D2}' => "implies",
        '\u{21D4}' => "if and only if",
        '\u{2225}' => "parallel to",
        '\u{22A5}' => "perpendicular to",
        '\u{2223}' => "divides",
        '\u{2211}' => "summation",
        '\u{220F}' => "product",
        '\u{2210}' => "coproduct",
        '\u{222B}' => "integral",
        '\u{222E}' => "contour integral",
        '\u{22C2}' => "intersection",
        '\u{22C3}' => "union",
        '\u{2A01}' => "direct sum",
        '\u{2A02}' => "tensor product",
        '\u{2A00}' => "circled dot",
        '\u{22C1}' => "logical or",
        '\u{22C0}' => "logical and",
        '\u{221E}' => "infinity",
        '\u{2202}' => "partial",
        '\u{2207}' => "nabla",
        '\u{221A}' => "square root",
        '\u{03B1}' => "alpha",
        '\u{03B2}' => "beta",
        '\u{03B3}' => "gamma",
        '\u{03B4}' => "delta",
        '\u{03B5}' => "epsilon",
        '\u{03B6}' => "zeta",
        '\u{03B7}' => "eta",
        '\u{03B8}' => "theta",
        '\u{03B9}' => "iota",
        '\u{03BA}' => "kappa",
        '\u{03BB}' => "lambda",
        '\u{03BC}' => "mu",
        '\u{03BD}' => "nu",
        '\u{03BE}' => "xi",
        '\u{03C0}' => "pi",
        '\u{03C1}' => "rho",
        '\u{03C3}' => "sigma",
        '\u{03C4}' => "tau",
        '\u{03C5}' => "upsilon",
        '\u{03C6}' => "phi",
        '\u{03C7}' => "chi",
        '\u{03C8}' => "psi",
        '\u{03C9}' => "omega",
        '\u{0393}' => "capital gamma",
        '\u{0394}' => "capital delta",
        '\u{0398}' => "capital theta",
        '\u{039B}' => "capital lambda",
        '\u{039E}' => "capital xi",
        '\u{03A0}' => "capital pi",
        '\u{03A3}' => "capital sigma",
        '\u{03A6}' => "capital phi",
        '\u{03A8}' => "capital psi",
        '\u{03A9}' => "capital omega",
        _ => return None,
    })
}

/// Spoken names for accent characters used in `\hat`, `\tilde`, and friends.
fn accent_name(ch: char) -> Option<&'static str> {
    Some(match ch {
        '^' | '\u{0302}' => "hat",
        '~' | '\u{0303}' => "tilde",
        '\u{00B4}' | '\u{0301}' => "acute",
        '`' | '\u{0300}' => "grave",
        '\u{00A8}' | '\u{0308}' => "dieresis",
        '.' | '\u{0307}' => "dot",
        '\u{00AF}' | '\u{0304}' => "bar",
        '\u{02D8}' | '\u{0306}' => "breve",
        '\u{02C7}' | '\u{030C}' => "check",
        '\u{2192}' | '\u{20D7}' => "vector",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use flashtex_math_layout::MathList;

    #[test]
    fn simple_relation_is_read_and_rendered_exactly() {
        let list = MathList::new(vec![
            Atom::ord('x'),
            Atom::bin('+'),
            Atom::ord('y'),
            Atom::rel('='),
            Atom::ord('z'),
        ]);
        let desc = MathAccessibility::new().describe(&list).unwrap();
        assert_eq!(desc.readable, "x plus y equals z");
        assert_eq!(
            desc.mathml,
            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\"><mrow><mi>x</mi><mo>+</mo><mi>y</mi><mo>=</mo><mi>z</mi></mrow></math>"
        );
        assert!(desc.is_fully_supported());
    }

    #[test]
    fn fraction_one_half_is_read_and_rendered_exactly() {
        let list = MathList::from(Atom::frac(MathList::symbols("1"), MathList::symbols("2")));
        let desc = MathAccessibility::new().describe(&list).unwrap();
        assert_eq!(desc.readable, "start fraction, 1, over, 2, end fraction");
        assert_eq!(
            desc.mathml,
            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\"><mrow><mfrac><mrow><mn>1</mn></mrow><mrow><mn>2</mn></mrow></mfrac></mrow></math>"
        );
        assert!(desc.is_fully_supported());
    }

    #[test]
    fn fraction_thickness_is_preserved_verbatim_in_mathml() {
        let mut atom = Atom::frac(MathList::symbols("1"), MathList::symbols("2"));
        if let Nucleus::Fraction { thickness, .. } = &mut atom.nucleus {
            *thickness = Some(0.0);
        }
        let desc = MathAccessibility::new()
            .describe(&MathList::from(atom))
            .unwrap();
        assert!(desc.mathml.contains("<mfrac linethickness=\"0pt\">"));
    }

    #[test]
    fn square_root_is_read_and_rendered_exactly() {
        let list = MathList::from(Atom::sqrt(MathList::symbols("2")));
        let desc = MathAccessibility::new().describe(&list).unwrap();
        assert_eq!(desc.readable, "start square root of 2, end square root");
        assert_eq!(
            desc.mathml,
            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\"><mrow><msqrt><mrow><mn>2</mn></mrow></msqrt></mrow></math>"
        );
        assert!(desc.is_fully_supported());
    }

    #[test]
    fn hat_accent_is_read_and_rendered_exactly() {
        let list = MathList::from(Atom::accent('^', MathList::symbols("x")));
        let desc = MathAccessibility::new().describe(&list).unwrap();
        assert_eq!(desc.readable, "x with hat accent");
        assert_eq!(
            desc.mathml,
            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\"><mrow><mover accent=\"true\"><mrow><mi>x</mi></mrow><mo>^</mo></mover></mrow></math>"
        );
        assert!(desc.is_fully_supported());
    }

    #[test]
    fn parenthesized_delimiter_is_read_and_rendered_exactly() {
        let list = MathList::from(Atom::left_right(
            Some('('),
            Some(')'),
            MathList::symbols("x"),
        ));
        let desc = MathAccessibility::new().describe(&list).unwrap();
        assert_eq!(desc.readable, "open parenthesis x close parenthesis");
        assert_eq!(
            desc.mathml,
            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\"><mrow><mrow><mo>(</mo><mrow><mi>x</mi></mrow><mo>)</mo></mrow></mrow></math>"
        );
        assert!(desc.is_fully_supported());
    }

    #[test]
    fn null_delimiter_omits_the_missing_side() {
        let list = MathList::from(Atom::left_right(None, Some('}'), MathList::symbols("x")));
        let desc = MathAccessibility::new().describe(&list).unwrap();
        assert_eq!(desc.readable, "x close brace");
        assert_eq!(
            desc.mathml,
            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\"><mrow><mrow><mrow><mi>x</mi></mrow><mo>}</mo></mrow></mrow></math>"
        );
    }

    #[test]
    fn superscript_alone_is_read_and_rendered_exactly() {
        let list = MathList::from(Atom::ord('x').with_sup(MathList::symbols("2")));
        let desc = MathAccessibility::new().describe(&list).unwrap();
        assert_eq!(desc.readable, "x superscript 2");
        assert_eq!(
            desc.mathml,
            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\"><mrow><msup><mi>x</mi><mrow><mn>2</mn></mrow></msup></mrow></math>"
        );
    }

    #[test]
    fn subscript_and_superscript_together_use_msubsup_order() {
        let list = MathList::from(
            Atom::ord('x')
                .with_sub(MathList::symbols("i"))
                .with_sup(MathList::symbols("2")),
        );
        let desc = MathAccessibility::new().describe(&list).unwrap();
        assert_eq!(desc.readable, "x subscript i superscript 2");
        assert_eq!(
            desc.mathml,
            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\"><mrow><msubsup><mi>x</mi><mrow><mi>i</mi></mrow><mrow><mn>2</mn></mrow></msubsup></mrow></math>"
        );
    }

    #[test]
    fn known_unicode_greek_letter_is_named_and_tagged_mi() {
        let list = MathList::from(Atom::ord('\u{03C0}'));
        let desc = MathAccessibility::new().describe(&list).unwrap();
        assert_eq!(desc.readable, "pi");
        assert_eq!(
            desc.mathml,
            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\"><mrow><mi>\u{03C0}</mi></mrow></math>"
        );
        assert!(desc.is_fully_supported());
    }

    #[test]
    fn unknown_unicode_symbol_is_reported_explicitly_not_guessed() {
        // U+1F600 GRINNING FACE has no semantic math name; we must not invent one.
        let list = MathList::from(Atom::ord('\u{1F600}'));
        let desc = MathAccessibility::new().describe(&list).unwrap();
        assert_eq!(desc.readable, "[unsupported symbol U+1F600]");
        assert_eq!(
            desc.mathml,
            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\"><mrow><mo>\u{1F600}</mo></mrow></math>"
        );
        assert_eq!(
            desc.unsupported,
            vec![Unsupported {
                path: vec![PathStep::Atom(0)],
                reason: UnsupportedReason::UnknownSymbolName('\u{1F600}'),
            }]
        );
        assert!(!desc.is_fully_supported());
    }

    #[test]
    fn control_character_becomes_an_explicit_merror_not_a_raw_byte() {
        // U+0007 BEL cannot legally appear as literal XML text.
        let list = MathList::from(Atom::ord('\u{0007}'));
        let desc = MathAccessibility::new().describe(&list).unwrap();
        assert_eq!(desc.readable, "[unsupported control character U+0007]");
        assert_eq!(
            desc.mathml,
            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\"><mrow><merror><mtext>unsupported control character U+0007</mtext></merror></mrow></math>"
        );
        assert_eq!(
            desc.unsupported,
            vec![Unsupported {
                path: vec![PathStep::Atom(0)],
                reason: UnsupportedReason::ControlCharacter('\u{0007}'),
            }]
        );
        assert!(!desc.mathml.chars().any(|c| c.is_control()));
    }

    #[test]
    fn xml_special_and_non_ascii_characters_round_trip_safely() {
        let list = MathList::new(vec![
            Atom::ord('<'),
            Atom::ord('&'),
            Atom::ord('\u{00E9}'), // 'e' with acute accent, not in the name table
        ]);
        let desc = MathAccessibility::new().describe(&list).unwrap();

        // Escaped correctly: no bare '<' or '&' outside of markup delimiters.
        assert_eq!(
            desc.mathml,
            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\"><mrow><mo>&lt;</mo><mo>&amp;</mo><mi>\u{00E9}</mi></mrow></math>"
        );

        // Round trip: decoding the entities and comparing against the source
        // characters recovers them exactly, including the non-ASCII one.
        let decoded = decode_xml_entities(&desc.mathml);
        assert!(decoded.contains('<'));
        assert!(decoded.contains('&'));
        assert!(decoded.contains('\u{00E9}'));

        // '&' has no spoken name; it must be reported, not guessed.
        assert!(
            desc.unsupported
                .iter()
                .any(|u| u.reason == UnsupportedReason::UnknownSymbolName('&'))
        );
        assert!(
            desc.unsupported
                .iter()
                .any(|u| u.reason == UnsupportedReason::UnknownSymbolName('\u{00E9}'))
        );
    }

    /// A minimal, test-only entity decoder used only to prove the round trip
    /// above; not part of the crate's public behavior.
    fn decode_xml_entities(s: &str) -> String {
        s.replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
    }

    #[test]
    fn deeply_nested_groups_fail_with_a_typed_error_not_a_stack_overflow() {
        // Built iteratively so constructing the fixture itself cannot overflow.
        let mut list = MathList::symbols("x");
        for _ in 0..10_000 {
            list = MathList::from(Atom::group(list));
        }
        let result = MathAccessibility::with_max_depth(50).describe(&list);
        match result {
            Err(AccessibilityError::RecursionLimitExceeded { max_depth, .. }) => {
                assert_eq!(max_depth, 50);
            }
            other => panic!("expected RecursionLimitExceeded, got {other:?}"),
        }
        // `MathList`/`Nucleus` (owned by math-layout, not ours to change) use
        // the compiler's default recursive drop glue, which would itself
        // overflow the stack walking 10,000 nested owned lists. That is a
        // fixture-teardown artifact, not the behavior under test (which is
        // that `describe` bails out via a typed error long before recursing
        // this deep) — leak the fixture instead of recursively dropping it.
        std::mem::forget(list);
    }

    #[test]
    fn nesting_within_the_bound_still_succeeds() {
        let mut list = MathList::symbols("x");
        for _ in 0..10 {
            list = MathList::from(Atom::group(list));
        }
        let desc = MathAccessibility::with_max_depth(50)
            .describe(&list)
            .unwrap();
        assert_eq!(desc.readable, "x");
    }

    #[test]
    fn empty_list_and_empty_nucleus_produce_no_text_and_a_bare_mrow() {
        let empty_list = MathList::new(vec![]);
        let desc = MathAccessibility::new().describe(&empty_list).unwrap();
        assert_eq!(desc.readable, "");
        assert_eq!(
            desc.mathml,
            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\"><mrow></mrow></math>"
        );

        let empty_nucleus = MathList::from(Atom::new(
            flashtex_math_layout::AtomClass::Ord,
            Nucleus::Empty,
        ));
        let desc = MathAccessibility::new().describe(&empty_nucleus).unwrap();
        assert_eq!(desc.readable, "");
    }

    #[test]
    fn escape_xml_text_escapes_only_the_three_reserved_characters() {
        assert_eq!(escape_xml_text("a<b&c>d"), "a&lt;b&amp;c&gt;d");
        assert_eq!(escape_xml_text("\u{03C0}"), "\u{03C0}");
    }
}
