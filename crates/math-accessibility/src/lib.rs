//! An original semantic accessibility adapter over `flashtex-math-layout`'s
//! math-list model ([`MathList`]/[`Atom`]/[`Nucleus`]).
//!
//! [`MathAccessibility::describe`] walks the *semantic* tree (not the laid-out
//! box tree) and produces:
//!
//! - `readable`: a structured, screen-reader-style spoken rendering.
//! - `mathml`: a standalone `<math>` document, correctly XML-escaped, that
//!   always preserves the exact source glyph identity of every symbol.
//! - `nodes`: every emitted description element, each carrying a [`NodeId`]
//!   that is an exact, stable identity for the source node it came from —
//!   derived purely from structural position in the input tree, never from
//!   memory addresses or traversal order, so it is reproducible across a
//!   rebuild of the same input (see the `identity_is_stable_across_rebuild`
//!   test).
//! - `unsupported`: an explicit, typed list of every node this adapter could
//!   not give a semantic name to. Nothing is silently skipped, and nothing is
//!   guessed: an unnamed symbol never gets a plausible-looking spoken word it
//!   was not actually given.
//!
//! Traversal is bounded two ways, both typed errors rather than a stack
//! overflow or unbounded memory use:
//! - nesting depth, via [`MathAccessibility::with_max_depth`]
//!   ([`DEFAULT_MAX_DEPTH`] by default) — bounds a deep-but-narrow tree;
//! - total node count, via [`MathAccessibility::with_max_nodes`]
//!   ([`DEFAULT_MAX_NODES`] by default) — bounds a wide-but-shallow tree.

use flashtex_math_layout::{Atom, MathList, Nucleus, StyleLevel};

/// Default bound on math-list nesting depth (see [`MathAccessibility::with_max_depth`]).
pub const DEFAULT_MAX_DEPTH: usize = 64;

/// Default bound on total nodes visited across the whole tree (see
/// [`MathAccessibility::with_max_nodes`]).
pub const DEFAULT_MAX_NODES: usize = 100_000;

/// One step of the path from the root [`MathList`] to a node. This is the
/// raw structural coordinate; [`NodeId`] is the identity built from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    /// Into a radical's optional degree (`\sqrt[degree]{...}`).
    Degree,
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
    /// Into an `\overline{...}` body.
    OverlineBody,
    /// Into an `\underline{...}` body.
    UnderlineBody,
    /// Into a style-override (`{\displaystyle ...}`) body.
    StyledBody,
}

/// A path from the root list to a specific node.
pub type NodePath = Vec<PathStep>;

/// The exact, stable identity of one source node.
///
/// It is built solely from the node's structural position in the input tree
/// (which atom index, which child slot at each level down from the root) —
/// never from memory addresses, insertion order, or any other incidental
/// detail of one particular run. Describing two structurally identical
/// inputs, built from scratch independently, always yields identical
/// `NodeId`s for corresponding nodes; see `identity_is_stable_across_rebuild`.
///
/// This is the value a consumer keeps to map a piece of speech or MathML
/// back to the exact node in the caller's own copy of the source tree.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(NodePath);

impl NodeId {
    /// The raw structural path this identity was built from.
    pub fn path(&self) -> &[PathStep] {
        &self.0
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            return write!(f, "root");
        }
        for (i, step) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, "/")?;
            }
            match step {
                PathStep::Atom(n) => write!(f, "atom{n}")?,
                PathStep::Group => write!(f, "group")?,
                PathStep::Superscript => write!(f, "superscript")?,
                PathStep::Subscript => write!(f, "subscript")?,
                PathStep::Numerator => write!(f, "numerator")?,
                PathStep::Denominator => write!(f, "denominator")?,
                PathStep::Radicand => write!(f, "radicand")?,
                PathStep::Degree => write!(f, "degree")?,
                PathStep::AccentBase => write!(f, "accent-base")?,
                PathStep::AccentGlyph => write!(f, "accent-glyph")?,
                PathStep::DelimitedBody => write!(f, "delimited-body")?,
                PathStep::LeftDelimiter => write!(f, "left-delimiter")?,
                PathStep::RightDelimiter => write!(f, "right-delimiter")?,
                PathStep::OverlineBody => write!(f, "overline-body")?,
                PathStep::UnderlineBody => write!(f, "underline-body")?,
                PathStep::StyledBody => write!(f, "styled-body")?,
            }
        }
        Ok(())
    }
}

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
    pub id: NodeId,
    pub reason: UnsupportedReason,
}

/// A typed failure. Every failure is a bounded-traversal condition:
/// everything else degrades to an explicit [`Unsupported`] marker instead of
/// failing outright.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessibilityError {
    /// Math-list nesting exceeded the configured depth bound.
    RecursionLimitExceeded { max_depth: usize, id: NodeId },
    /// The total number of nodes visited exceeded the configured bound
    /// (guards a wide-but-shallow tree, which a depth bound alone cannot).
    NodeCountExceeded { max_nodes: usize, id: NodeId },
}

impl std::fmt::Display for AccessibilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessibilityError::RecursionLimitExceeded { max_depth, id } => write!(
                f,
                "math list nesting exceeded the bound of {max_depth} level(s) at {id}"
            ),
            AccessibilityError::NodeCountExceeded { max_nodes, id } => write!(
                f,
                "math list node count exceeded the bound of {max_nodes} node(s) at {id}"
            ),
        }
    }
}

impl std::error::Error for AccessibilityError {}

/// One emitted description element: the readable text produced for exactly
/// one source node, paired with that node's exact identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DescribedNode {
    pub id: NodeId,
    pub readable: String,
}

/// The result of describing a [`MathList`].
#[derive(Debug, Clone, PartialEq)]
pub struct Description {
    /// A structured, spoken-word rendering of the whole list.
    pub readable: String,
    /// A standalone, XML-escaped `<math>...</math>` document.
    pub mathml: String,
    /// Every emitted node, in traversal (pre-order) order, each carrying the
    /// exact identity of the source node it came from.
    pub nodes: Vec<DescribedNode>,
    /// Every node that could not be given a semantic name, in traversal order.
    pub unsupported: Vec<Unsupported>,
}

impl Description {
    /// `true` if every node in the tree was understood.
    pub fn is_fully_supported(&self) -> bool {
        self.unsupported.is_empty()
    }
}

/// A semantic math-accessibility adapter, configured with traversal bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MathAccessibility {
    max_depth: usize,
    max_nodes: usize,
}

impl Default for MathAccessibility {
    fn default() -> Self {
        Self::new()
    }
}

impl MathAccessibility {
    /// A new adapter with [`DEFAULT_MAX_DEPTH`] and [`DEFAULT_MAX_NODES`].
    pub fn new() -> Self {
        MathAccessibility {
            max_depth: DEFAULT_MAX_DEPTH,
            max_nodes: DEFAULT_MAX_NODES,
        }
    }

    /// A new adapter with an explicit nesting-depth bound and the default
    /// node-count bound.
    pub fn with_max_depth(max_depth: usize) -> Self {
        MathAccessibility {
            max_depth,
            max_nodes: DEFAULT_MAX_NODES,
        }
    }

    /// A new adapter with an explicit total-node-count bound and the default
    /// depth bound.
    pub fn with_max_nodes(max_nodes: usize) -> Self {
        MathAccessibility {
            max_depth: DEFAULT_MAX_DEPTH,
            max_nodes,
        }
    }

    /// A new adapter with both bounds set explicitly.
    pub fn with_bounds(max_depth: usize, max_nodes: usize) -> Self {
        MathAccessibility {
            max_depth,
            max_nodes,
        }
    }

    /// Describes `list`, returning structured readable text, MathML, per-node
    /// identities, and explicit unsupported markers — or a typed error if the
    /// list exceeds this adapter's depth or node-count bound.
    pub fn describe(&self, list: &MathList) -> Result<Description, AccessibilityError> {
        let mut renderer = Renderer {
            max_depth: self.max_depth,
            max_nodes: self.max_nodes,
            nodes_visited: 0,
            unsupported: Vec::new(),
            nodes: Vec::new(),
        };
        let mut path = Vec::new();
        let rendered = renderer.render_list(list, 0, &mut path)?;
        Ok(Description {
            readable: rendered.readable,
            mathml: format!(
                "<math xmlns=\"http://www.w3.org/1998/Math/MathML\">{}</math>",
                rendered.mathml
            ),
            nodes: renderer.nodes,
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
    max_nodes: usize,
    nodes_visited: usize,
    unsupported: Vec<Unsupported>,
    nodes: Vec<DescribedNode>,
}

impl Renderer {
    fn check_depth(&self, depth: usize, path: &[PathStep]) -> Result<(), AccessibilityError> {
        if depth > self.max_depth {
            Err(AccessibilityError::RecursionLimitExceeded {
                max_depth: self.max_depth,
                id: NodeId(path.to_vec()),
            })
        } else {
            Ok(())
        }
    }

    /// Records that one more node has been visited, bounding total node
    /// count independently of nesting depth so a wide-but-shallow tree
    /// cannot blow up even at `depth == 1`.
    fn check_node_count(&mut self, path: &[PathStep]) -> Result<(), AccessibilityError> {
        self.nodes_visited += 1;
        if self.nodes_visited > self.max_nodes {
            Err(AccessibilityError::NodeCountExceeded {
                max_nodes: self.max_nodes,
                id: NodeId(path.to_vec()),
            })
        } else {
            Ok(())
        }
    }

    /// Records an emitted description element for the node at `path`.
    fn record(&mut self, path: &[PathStep], readable: &str) {
        self.nodes.push(DescribedNode {
            id: NodeId(path.to_vec()),
            readable: readable.to_string(),
        });
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
            if let Err(err) = self.check_node_count(path) {
                path.pop();
                return Err(err);
            }
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
                self.record(path, &sup.readable);
                path.pop();
                Ok(Rendered {
                    readable: format!("{} superscript {}", nucleus.readable, sup.readable),
                    mathml: format!("<msup>{}{}</msup>", nucleus.mathml, sup.mathml),
                })
            }
            (None, Some(sub)) => {
                path.push(PathStep::Subscript);
                let sub = self.render_list(sub, depth + 1, path)?;
                self.record(path, &sub.readable);
                path.pop();
                Ok(Rendered {
                    readable: format!("{} subscript {}", nucleus.readable, sub.readable),
                    mathml: format!("<msub>{}{}</msub>", nucleus.mathml, sub.mathml),
                })
            }
            (Some(sup), Some(sub)) => {
                path.push(PathStep::Subscript);
                let sub = self.render_list(sub, depth + 1, path)?;
                self.record(path, &sub.readable);
                path.pop();
                path.push(PathStep::Superscript);
                let sup = self.render_list(sup, depth + 1, path)?;
                self.record(path, &sup.readable);
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
        let rendered = match nucleus {
            Nucleus::Symbol(ch) => self.render_symbol(*ch, path),
            Nucleus::List(list) => {
                path.push(PathStep::Group);
                let rendered = self.render_list(list, depth + 1, path)?;
                path.pop();
                rendered
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
                Rendered {
                    readable: format!(
                        "start fraction, {}, over, {}, end fraction",
                        num.readable, den.readable
                    ),
                    mathml,
                }
            }
            Nucleus::Radical { radicand, degree } => {
                path.push(PathStep::Radicand);
                let radicand = self.render_list(radicand, depth + 1, path)?;
                path.pop();
                match degree {
                    None => Rendered {
                        readable: format!(
                            "start square root of {}, end square root",
                            radicand.readable
                        ),
                        mathml: format!("<msqrt>{}</msqrt>", radicand.mathml),
                    },
                    Some(degree) => {
                        path.push(PathStep::Degree);
                        let degree = self.render_list(degree, depth + 1, path)?;
                        path.pop();
                        Rendered {
                            readable: format!(
                                "start root, index {}, {}, end root",
                                degree.readable, radicand.readable
                            ),
                            // MathML mroot child order is (radicand, index).
                            mathml: format!("<mroot>{}{}</mroot>", radicand.mathml, degree.mathml),
                        }
                    }
                }
            }
            Nucleus::Text(text) => self.render_text(text, path),
            Nucleus::Overline(body) => {
                path.push(PathStep::OverlineBody);
                let body = self.render_list(body, depth + 1, path)?;
                path.pop();
                Rendered {
                    readable: format!("start overline, {}, end overline", body.readable),
                    mathml: format!(
                        "<mover accent=\"true\">{}<mo>\u{00AF}</mo></mover>",
                        body.mathml
                    ),
                }
            }
            Nucleus::Underline(body) => {
                path.push(PathStep::UnderlineBody);
                let body = self.render_list(body, depth + 1, path)?;
                path.pop();
                Rendered {
                    readable: format!("start underline, {}, end underline", body.readable),
                    mathml: format!("<munder accent=\"true\">{}<mo>_</mo></munder>", body.mathml),
                }
            }
            Nucleus::Styled { style, body } => {
                path.push(PathStep::StyledBody);
                let body = self.render_list(body, depth + 1, path)?;
                path.pop();
                // The style's exact level is carried verbatim as MathML's
                // `displaystyle` attribute rather than dropped or guessed at.
                let displaystyle = style.level == StyleLevel::Display;
                Rendered {
                    readable: body.readable,
                    mathml: format!(
                        "<mstyle displaystyle=\"{displaystyle}\">{}</mstyle>",
                        body.mathml
                    ),
                }
            }
            Nucleus::Accent { accent, base } => {
                path.push(PathStep::AccentBase);
                let base = self.render_list(base, depth + 1, path)?;
                path.pop();
                path.push(PathStep::AccentGlyph);
                let (accent_word, accent_mathml) = self.render_accent_glyph(*accent, path);
                path.pop();
                Rendered {
                    readable: format!("{} with {} accent", base.readable, accent_word),
                    mathml: format!(
                        "<mover accent=\"true\">{}{}</mover>",
                        base.mathml, accent_mathml
                    ),
                }
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
                Rendered {
                    readable,
                    mathml: format!(
                        "<mrow>{}{}{}</mrow>",
                        left.mathml, body.mathml, right.mathml
                    ),
                }
            }
            Nucleus::Empty => Rendered {
                readable: String::new(),
                mathml: "<mrow></mrow>".to_string(),
            },
        };
        self.record(path, &rendered.readable);
        Ok(rendered)
    }

    /// Renders a single symbol character. Never guesses: a character with no
    /// spoken-name entry becomes an explicit `[unsupported symbol U+XXXX]`
    /// marker recorded in `self.unsupported`, and control characters (which
    /// cannot legally appear as literal MathML text) become an explicit
    /// `<merror>` marker instead of a raw byte.
    fn render_symbol(&mut self, ch: char, path: &[PathStep]) -> Rendered {
        if ch.is_control() {
            self.unsupported.push(Unsupported {
                id: NodeId(path.to_vec()),
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
                id: NodeId(path.to_vec()),
                reason: UnsupportedReason::UnknownSymbolName(ch),
            });
            format!("[unsupported symbol U+{:04X}]", ch as u32)
        };
        Rendered { readable, mathml }
    }

    /// Renders `\lim`, `\sin`, and similar upright-text nuclei. The text is
    /// spoken exactly as given (it is already a name, not a symbol we would
    /// have to guess a name for); a control character embedded in it is
    /// still reported explicitly rather than emitted as a raw byte.
    fn render_text(&mut self, text: &str, path: &[PathStep]) -> Rendered {
        if let Some(bad) = text.chars().find(|c| c.is_control()) {
            self.unsupported.push(Unsupported {
                id: NodeId(path.to_vec()),
                reason: UnsupportedReason::ControlCharacter(bad),
            });
            let note = format!("unsupported control character U+{:04X}", bad as u32);
            return Rendered {
                readable: format!("[{note}]"),
                mathml: format!("<merror><mtext>{note}</mtext></merror>"),
            };
        }
        Rendered {
            readable: text.to_string(),
            mathml: format!("<mtext>{}</mtext>", escape_xml_text(text)),
        }
    }

    fn render_accent_glyph(&mut self, accent: char, path: &[PathStep]) -> (String, String) {
        if accent.is_control() {
            self.unsupported.push(Unsupported {
                id: NodeId(path.to_vec()),
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
                    id: NodeId(path.to_vec()),
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
    use flashtex_math_layout::{MathList, Style};

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
    fn nth_root_with_degree_is_read_and_rendered_exactly() {
        let list = MathList::from(Atom::root(MathList::symbols("3"), MathList::symbols("8")));
        let desc = MathAccessibility::new().describe(&list).unwrap();
        assert_eq!(desc.readable, "start root, index 3, 8, end root");
        assert_eq!(
            desc.mathml,
            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\"><mrow><mroot><mrow><mn>8</mn></mrow><mrow><mn>3</mn></mrow></mroot></mrow></math>"
        );
        assert!(desc.is_fully_supported());
    }

    #[test]
    fn text_operator_is_spoken_as_given_not_guessed() {
        let list = MathList::from(Atom::text_op("lim"));
        let desc = MathAccessibility::new().describe(&list).unwrap();
        assert_eq!(desc.readable, "lim");
        assert_eq!(
            desc.mathml,
            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\"><mrow><mtext>lim</mtext></mrow></math>"
        );
        assert!(desc.is_fully_supported());
    }

    #[test]
    fn overline_and_underline_are_read_and_rendered_exactly() {
        let over = MathList::from(Atom::overline(MathList::symbols("x")));
        let desc = MathAccessibility::new().describe(&over).unwrap();
        assert_eq!(desc.readable, "start overline, x, end overline");
        assert!(desc.mathml.contains("<mover accent=\"true\">"));

        let under = MathList::from(Atom::underline(MathList::symbols("x")));
        let desc = MathAccessibility::new().describe(&under).unwrap();
        assert_eq!(desc.readable, "start underline, x, end underline");
        assert!(desc.mathml.contains("<munder accent=\"true\">"));
    }

    #[test]
    fn styled_body_carries_displaystyle_verbatim_and_reads_transparently() {
        let list = MathList::from(Atom::styled(Style::DISPLAY, MathList::symbols("x")));
        let desc = MathAccessibility::new().describe(&list).unwrap();
        assert_eq!(desc.readable, "x");
        assert!(desc.mathml.contains("<mstyle displaystyle=\"true\">"));

        let list = MathList::from(Atom::styled(Style::TEXT, MathList::symbols("x")));
        let desc = MathAccessibility::new().describe(&list).unwrap();
        assert!(desc.mathml.contains("<mstyle displaystyle=\"false\">"));
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
        assert_eq!(desc.unsupported.len(), 1);
        assert_eq!(desc.unsupported[0].id.path(), &[PathStep::Atom(0)]);
        assert_eq!(
            desc.unsupported[0].reason,
            UnsupportedReason::UnknownSymbolName('\u{1F600}')
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
        assert_eq!(desc.unsupported.len(), 1);
        assert_eq!(desc.unsupported[0].id.path(), &[PathStep::Atom(0)]);
        assert_eq!(
            desc.unsupported[0].reason,
            UnsupportedReason::ControlCharacter('\u{0007}')
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
    fn wide_shallow_list_fails_on_node_count_not_depth() {
        // 200 atoms in one flat list: depth stays at 1 throughout, so only a
        // total-node-count bound (not the depth bound) can catch this.
        let list = MathList::symbols(&"x".repeat(200));
        let result = MathAccessibility::with_max_nodes(100).describe(&list);
        match result {
            Err(AccessibilityError::NodeCountExceeded { max_nodes, id }) => {
                assert_eq!(max_nodes, 100);
                // The 101st atom (index 100) is exactly where the bound trips.
                assert_eq!(id.path(), &[PathStep::Atom(100)]);
            }
            other => panic!("expected NodeCountExceeded, got {other:?}"),
        }
    }

    #[test]
    fn wide_shallow_list_within_the_node_bound_still_succeeds() {
        let list = MathList::symbols(&"x".repeat(50));
        let desc = MathAccessibility::with_max_nodes(100)
            .describe(&list)
            .unwrap();
        assert_eq!(desc.readable, vec!["x"; 50].join(" "));
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

    /// Builds a moderately nested, mixed-content fixture from scratch. Called
    /// twice independently in `identity_is_stable_across_rebuild` to prove
    /// `NodeId`s are structural, not tied to one particular allocation.
    fn build_fixture() -> MathList {
        let frac = Atom::frac(
            MathList::from(Atom::ord('\u{1F600}')), // unsupported: forces an id into `unsupported`
            MathList::symbols("2"),
        );
        let root = Atom::accent('^', MathList::from(frac));
        MathList::new(vec![Atom::ord('x'), root.with_sup(MathList::symbols("n"))])
    }

    #[test]
    fn identity_is_stable_across_rebuild() {
        // Two independently constructed trees with the same structure and
        // content — not the same allocation, not the same `Vec`/`String`
        // instances.
        let tree_a = build_fixture();
        let tree_b = build_fixture();
        assert_ne!(
            tree_a.atoms.as_ptr(),
            tree_b.atoms.as_ptr(),
            "fixture must actually be rebuilt, not reused, for this test to prove anything"
        );

        let desc_a = MathAccessibility::new().describe(&tree_a).unwrap();
        let desc_b = MathAccessibility::new().describe(&tree_b).unwrap();

        // Same emitted nodes, in the same order, with identical ids and text.
        assert_eq!(desc_a.nodes.len(), desc_b.nodes.len());
        assert!(!desc_a.nodes.is_empty());
        for (a, b) in desc_a.nodes.iter().zip(desc_b.nodes.iter()) {
            assert_eq!(a.id, b.id);
            assert_eq!(a.readable, b.readable);
            assert_eq!(a.id.to_string(), b.id.to_string());
        }

        // Same unsupported markers, at the same identity.
        assert_eq!(desc_a.unsupported, desc_b.unsupported);
        assert!(!desc_a.unsupported.is_empty());

        assert_eq!(desc_a.readable, desc_b.readable);
        assert_eq!(desc_a.mathml, desc_b.mathml);
    }

    #[test]
    fn node_id_display_is_a_stable_readable_path() {
        let list = MathList::from(Atom::frac(MathList::symbols("1"), MathList::symbols("2")));
        let desc = MathAccessibility::new().describe(&list).unwrap();
        // The numerator "1" atom lives at atom0/numerator/atom0.
        let numerator_leaf = desc
            .nodes
            .iter()
            .find(|n| n.id.to_string() == "atom0/numerator/atom0")
            .expect("numerator leaf node must be recorded with this exact id");
        assert_eq!(numerator_leaf.readable, "1");
    }

    // =========================================================================
    // FT-038 rev3: bounded adversarial acceptance suite.
    //
    // Each case below attacks `describe` with a malformed, oversized, or
    // hostile input and asserts the *outcome type*: either a typed
    // `AccessibilityError` at an exact, predetermined boundary, or a
    // `Description` whose `mathml` is provably well-formed and cannot be
    // broken out of. None of these may panic, hang, or overflow the stack.
    // =========================================================================

    /// The only element tags this crate ever emits. Used by
    /// `assert_no_foreign_markup` to prove hostile input cannot inject a
    /// foreign tag, attribute, comment, CDATA section, or processing
    /// instruction into the output.
    const KNOWN_TAGS: &[&str] = &[
        "math", "mrow", "mi", "mo", "mn", "mtext", "mfrac", "msqrt", "mroot", "msup", "msub",
        "msubsup", "mover", "munder", "mstyle", "merror",
    ];

    /// Walks `mathml` byte-by-byte and panics if any literal `<` does not
    /// begin one of `KNOWN_TAGS` (as an opening or closing tag). This is the
    /// proof that hostile text can never break out of the document
    /// structure: `escape_xml_text` guarantees every `<` that comes from
    /// *content* is rewritten to `&lt;` before it ever reaches the output,
    /// so the only literal `<` bytes that can survive are the ones this
    /// crate's own renderer writes as markup.
    fn assert_no_foreign_markup(mathml: &str) {
        let bytes = mathml.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'<' {
                let mut j = i + 1;
                if j < bytes.len() && bytes[j] == b'/' {
                    j += 1;
                }
                let rest = &mathml[j..];
                let matched = KNOWN_TAGS.iter().any(|tag| {
                    rest.starts_with(tag)
                        && matches!(rest[tag.len()..].chars().next(), Some('>') | Some(' '))
                });
                assert!(
                    matched,
                    "found a '<' at byte {i} not starting a known element tag: {:?}",
                    &mathml[i..(i + 20).min(mathml.len())]
                );
            }
            i += 1;
        }
    }

    /// Wraps `leaf` in `n` nested single-atom braced groups
    /// (`Nucleus::List`), so that rendering `leaf` itself happens at
    /// exactly nesting depth `n`.
    fn nested_single_atom_groups(n: usize, leaf: MathList) -> MathList {
        let mut list = leaf;
        for _ in 0..n {
            list = MathList::from(Atom::group(list));
        }
        list
    }

    /// Like `nested_single_atom_groups`, but returns the single wrapping
    /// `Atom` instead of the one-atom `MathList` that contains it, so it can
    /// be embedded as one sibling among other atoms (for "wide and deep"
    /// fixtures). Requires `n >= 1`.
    fn deep_group_atom(n: usize, leaf: MathList) -> Atom {
        assert!(n >= 1);
        nested_single_atom_groups(n, leaf)
            .atoms
            .into_iter()
            .next()
            .expect("n >= 1 wraps produce exactly one atom")
    }

    #[test]
    fn depth_exactly_at_bound_succeeds() {
        let list = nested_single_atom_groups(50, MathList::symbols("x"));
        let desc = MathAccessibility::with_max_depth(50)
            .describe(&list)
            .unwrap();
        assert_eq!(desc.readable, "x");
    }

    #[test]
    fn depth_one_past_bound_fails() {
        let list = nested_single_atom_groups(51, MathList::symbols("x"));
        let result = MathAccessibility::with_max_depth(50).describe(&list);
        match result {
            Err(AccessibilityError::RecursionLimitExceeded { max_depth, .. }) => {
                assert_eq!(max_depth, 50);
            }
            other => panic!("expected RecursionLimitExceeded, got {other:?}"),
        }
    }

    #[test]
    fn node_count_exactly_at_bound_succeeds() {
        let list = MathList::symbols(&"x".repeat(100));
        let desc = MathAccessibility::with_max_nodes(100)
            .describe(&list)
            .unwrap();
        assert_eq!(desc.nodes.len(), 100);
    }

    #[test]
    fn node_count_one_past_bound_fails() {
        let list = MathList::symbols(&"x".repeat(101));
        let result = MathAccessibility::with_max_nodes(100).describe(&list);
        match result {
            Err(AccessibilityError::NodeCountExceeded { max_nodes, id }) => {
                assert_eq!(max_nodes, 100);
                assert_eq!(id.path(), &[PathStep::Atom(100)]);
            }
            other => panic!("expected NodeCountExceeded, got {other:?}"),
        }
    }

    #[test]
    fn wide_and_deep_tree_trips_depth_bound_before_node_count_bound() {
        // 40 wide siblings at the top level (cheap against a node-count
        // budget of 1000), plus one branch nested far past a depth budget
        // of 5. Node count at the moment of failure is nowhere near 1000 --
        // only the depth bound can be responsible.
        let mut atoms: Vec<Atom> = (0..40).map(|_| Atom::ord('x')).collect();
        atoms.push(deep_group_atom(20, MathList::symbols("y")));
        let list = MathList::new(atoms);
        let result = MathAccessibility::with_bounds(5, 1000).describe(&list);
        match result {
            Err(AccessibilityError::RecursionLimitExceeded { max_depth, .. }) => {
                assert_eq!(max_depth, 5);
            }
            other => panic!("expected RecursionLimitExceeded, got {other:?}"),
        }
    }

    #[test]
    fn wide_and_deep_tree_trips_node_count_bound_before_depth_bound() {
        // 49 wide siblings followed by one branch nested 10 levels deep --
        // far under a depth budget of 1000. Node count exceeds its budget
        // of 30 while still walking the wide prefix, long before traversal
        // ever reaches the deep branch.
        let mut atoms: Vec<Atom> = (0..49).map(|_| Atom::ord('x')).collect();
        atoms.push(deep_group_atom(10, MathList::symbols("y")));
        let list = MathList::new(atoms);
        let result = MathAccessibility::with_bounds(1000, 30).describe(&list);
        match result {
            Err(AccessibilityError::NodeCountExceeded { max_nodes, id }) => {
                assert_eq!(max_nodes, 30);
                // Tripped on the 31st top-level atom; traversal never even
                // reached the deep branch at index 49.
                assert_eq!(id.path(), &[PathStep::Atom(30)]);
            }
            other => panic!("expected NodeCountExceeded, got {other:?}"),
        }
    }

    #[test]
    fn empty_document_succeeds_even_with_zero_bounds() {
        let empty = MathList::new(vec![]);
        let desc = MathAccessibility::with_bounds(0, 0)
            .describe(&empty)
            .unwrap();
        assert_eq!(desc.readable, "");
        assert_eq!(
            desc.mathml,
            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\"><mrow></mrow></math>"
        );
        assert!(desc.nodes.is_empty());
        assert!(desc.unsupported.is_empty());
        assert_no_foreign_markup(&desc.mathml);
    }

    #[test]
    fn node_with_all_empty_children_does_not_panic_and_stays_well_formed() {
        // A fraction whose numerator AND denominator are both empty lists:
        // a node whose children are all empty.
        let atom = Atom::frac(MathList::new(vec![]), MathList::new(vec![]));
        let desc = MathAccessibility::new()
            .describe(&MathList::from(atom))
            .unwrap();
        assert_eq!(desc.readable, "start fraction, , over, , end fraction");
        assert!(
            desc.mathml
                .contains("<mfrac><mrow></mrow><mrow></mrow></mfrac>")
        );
        assert_no_foreign_markup(&desc.mathml);
    }

    #[test]
    fn assorted_control_characters_degrade_to_merror_not_a_raw_byte() {
        // NUL, other C0 controls, DEL, and a C1 control all take the same
        // safe path as the BEL case covered above: an explicit <merror>
        // marker, never a literal control byte in the output.
        for ch in ['\u{0000}', '\u{0001}', '\u{001B}', '\u{007F}', '\u{0080}'] {
            assert!(ch.is_control());
            let list = MathList::from(Atom::ord(ch));
            let desc = MathAccessibility::new().describe(&list).unwrap();
            assert!(!desc.mathml.chars().any(|c| c.is_control()));
            assert_eq!(desc.unsupported.len(), 1);
            assert_eq!(
                desc.unsupported[0].reason,
                UnsupportedReason::ControlCharacter(ch)
            );
            assert_no_foreign_markup(&desc.mathml);
        }
    }

    #[test]
    fn right_to_left_override_is_not_treated_as_a_control_character() {
        // U+202E RIGHT-TO-LEFT OVERRIDE is a Unicode formatting character
        // (general category Cf), not a control character (Cc): it must not
        // be silently swallowed by the control-character path. It also has
        // no spoken name, so it must not be guessed at -- it becomes an
        // explicit unsupported marker, with its exact glyph preserved
        // (never stripped) in the MathML.
        let rlo = '\u{202E}';
        assert!(!rlo.is_control());
        let list = MathList::from(Atom::ord(rlo));
        let desc = MathAccessibility::new().describe(&list).unwrap();
        assert_eq!(desc.readable, "[unsupported symbol U+202E]");
        assert!(desc.mathml.contains(&format!("<mo>{rlo}</mo>")));
        assert_eq!(desc.unsupported.len(), 1);
        assert_eq!(
            desc.unsupported[0].reason,
            UnsupportedReason::UnknownSymbolName(rlo)
        );
        assert_no_foreign_markup(&desc.mathml);
    }

    #[test]
    fn unpaired_surrogate_code_points_cannot_be_constructed_as_a_char() {
        // Rust's `char` type statically excludes the UTF-16 surrogate range
        // (U+D800..=U+DFFF); there is no way to hand this crate an
        // "unpaired surrogate" in the first place -- the type system is the
        // bound. We prove the fence exists, and that the legal values on
        // either side of it are ordinary, safely handled symbols.
        assert!(char::from_u32(0xD800).is_none());
        assert!(char::from_u32(0xDFFF).is_none());

        let just_below = char::from_u32(0xD7FF).unwrap();
        let just_above = char::from_u32(0xE000).unwrap();
        let list = MathList::new(vec![Atom::ord(just_below), Atom::ord(just_above)]);
        let desc = MathAccessibility::new().describe(&list).unwrap();
        assert_no_foreign_markup(&desc.mathml);
        // Neither is ASCII alphanumeric nor in the named-symbol table, so
        // both are explicit unsupported markers -- never a guess, never a
        // panic.
        assert_eq!(desc.unsupported.len(), 2);
    }

    #[test]
    fn assorted_unknown_symbols_are_reported_not_guessed() {
        let unknowns = ['\u{1F600}', '\u{2603}', '\u{E000}', '\u{FFFD}'];
        for ch in unknowns {
            let list = MathList::from(Atom::ord(ch));
            let desc = MathAccessibility::new().describe(&list).unwrap();
            assert_eq!(desc.unsupported.len(), 1);
            assert_eq!(
                desc.unsupported[0].reason,
                UnsupportedReason::UnknownSymbolName(ch)
            );
            assert_no_foreign_markup(&desc.mathml);
        }
    }

    #[test]
    fn hostile_text_with_full_tag_injection_cannot_break_out_of_mathml() {
        // Each payload is a classic XML/HTML injection attempt: closing the
        // enclosing element early, opening a new element, a comment, a
        // CDATA section, a processing instruction, and a DOCTYPE with an
        // external entity. None of these may produce foreign markup, and
        // none may be silently dropped -- decoding the entities back out
        // must recover the payload byte-for-byte.
        let payloads = [
            "</mtext></mrow></math><script>alert(1)</script>",
            "\"><img src=x onerror=alert(1)>",
            "<!--",
            "-->",
            "<![CDATA[",
            "]]>",
            "<?xml version=\"1.0\"?>",
            "<!DOCTYPE math [<!ENTITY x SYSTEM \"file:///etc/passwd\">]>",
            "</math><math>injected</math><math>",
        ];
        for payload in payloads {
            let list = MathList::from(Atom::text_op(payload));
            let desc = MathAccessibility::new().describe(&list).unwrap();
            assert_no_foreign_markup(&desc.mathml);
            let escaped = escape_xml_text(payload);
            assert!(
                desc.mathml.contains(&format!("<mtext>{escaped}</mtext>")),
                "payload {payload:?} must be preserved, only escaped: {}",
                desc.mathml
            );
            let decoded = decode_xml_entities(&desc.mathml);
            assert!(decoded.contains(payload));
        }
    }

    #[test]
    fn control_character_mixed_into_hostile_text_still_degrades_safely() {
        // When a control character appears anywhere in an upright-text
        // nucleus the whole node degrades to a single <merror> marker (see
        // `render_text`) -- a stricter degradation than symbol-by-symbol
        // escaping, but still bounded and typed: never a panic, never a raw
        // control byte or an unescaped tag fragment in the output.
        let hostile = "<script>\u{0}</script>";
        let list = MathList::from(Atom::text_op(hostile));
        let desc = MathAccessibility::new().describe(&list).unwrap();
        assert_no_foreign_markup(&desc.mathml);
        assert!(!desc.mathml.chars().any(|c| c.is_control()));
        assert_eq!(desc.unsupported.len(), 1);
    }

    // =========================================================================
    // FT-038 rev3: node-identity acceptance suite.
    //
    // A specification of the identity contract: `NodeId` is derived purely
    // from a node's structural position in the input tree, so --
    //   1. identical structures, rebuilt independently, get identical ids;
    //   2. a structurally different tree gets a different id at the node
    //      whose position changed;
    //   3. reordering siblings changes the ids of exactly the moved nodes
    //      (identity tracks position, not content).
    // =========================================================================

    #[test]
    fn identity_spec_identical_structures_produce_identical_ids() {
        let build = || MathList::new(vec![Atom::ord('a'), Atom::bin('+'), Atom::ord('b')]);
        let a = MathAccessibility::new().describe(&build()).unwrap();
        let b = MathAccessibility::new().describe(&build()).unwrap();
        let ids_a: Vec<_> = a.nodes.iter().map(|n| n.id.clone()).collect();
        let ids_b: Vec<_> = b.nodes.iter().map(|n| n.id.clone()).collect();
        assert_eq!(ids_a, ids_b);
        assert!(!ids_a.is_empty());
    }

    #[test]
    fn identity_spec_structurally_different_tree_produces_different_ids() {
        let flat = MathList::new(vec![Atom::ord('a'), Atom::ord('b')]);
        let nested = MathList::new(vec![
            Atom::group(MathList::from(Atom::ord('a'))),
            Atom::ord('b'),
        ]);

        let flat_desc = MathAccessibility::new().describe(&flat).unwrap();
        let nested_desc = MathAccessibility::new().describe(&nested).unwrap();

        // 'a' sits at atom0 in the flat tree, but at atom0/group/atom0 once
        // wrapped in a group: same content, different structural identity.
        let flat_a_id = flat_desc
            .nodes
            .iter()
            .find(|n| n.readable == "a")
            .unwrap()
            .id
            .to_string();
        let nested_a_id = nested_desc
            .nodes
            .iter()
            .find(|n| n.readable == "a")
            .unwrap()
            .id
            .to_string();
        assert_eq!(flat_a_id, "atom0");
        assert_eq!(nested_a_id, "atom0/group/atom0");
        assert_ne!(flat_a_id, nested_a_id);
    }

    #[test]
    fn identity_spec_reordering_siblings_changes_the_moved_nodes_ids() {
        let original = MathList::new(vec![Atom::ord('a'), Atom::ord('b'), Atom::ord('c')]);
        let reordered = MathList::new(vec![Atom::ord('c'), Atom::ord('a'), Atom::ord('b')]);

        let d1 = MathAccessibility::new().describe(&original).unwrap();
        let d2 = MathAccessibility::new().describe(&reordered).unwrap();

        let id_of = |desc: &Description, readable: &str| {
            desc.nodes
                .iter()
                .find(|n| n.readable == readable)
                .unwrap()
                .id
                .to_string()
        };

        // 'a' moves from position 0 to position 1: its id must move with it.
        assert_eq!(id_of(&d1, "a"), "atom0");
        assert_eq!(id_of(&d2, "a"), "atom1");
        assert_ne!(id_of(&d1, "a"), id_of(&d2, "a"));

        // 'c' moves from position 2 to position 0.
        assert_eq!(id_of(&d1, "c"), "atom2");
        assert_eq!(id_of(&d2, "c"), "atom0");
        assert_ne!(id_of(&d1, "c"), id_of(&d2, "c"));

        // 'b' stays put at position 1... except position 1 is now occupied
        // by 'a', so 'b' itself moved to position 2 -- confirming identity
        // tracks the slot's occupant, not a fixed label.
        assert_eq!(id_of(&d1, "b"), "atom1");
        assert_eq!(id_of(&d2, "b"), "atom2");
    }
}
