//! unicode-math: math alphabets as Unicode Mathematical Alphanumeric
//! Symbols, the `math-style` / `bold-style` / `sans-style` options, and the
//! `\math..`/`\sym..` alphabet commands.
//!
//! Source: `unicode-math-xetex.sty` (TeX Live 2026), checked against the
//! characters XeLaTeX and LuaLaTeX put in the PDF (`fixtures/expected/math_*`).

use crate::keyval::{self, KeyVal};

/// A math alphabet (Unicode "mathematical" style).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Alphabet {
    Up,
    It,
    Bf,
    BfIt,
    Sf,
    SfIt,
    BfSf,
    BfSfIt,
    Tt,
    Bb,
    BbIt,
    Scr,
    BfScr,
    Frak,
    BfFrak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalStyle {
    Iso,
    Tex,
    French,
    Upright,
    Literal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoldStyle {
    Iso,
    Tex,
    Upright,
    Literal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SansStyle {
    Italic,
    Upright,
    Literal,
}

/// Resolved unicode-math options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MathOptions {
    pub normal: NormalStyle,
    pub bold: BoldStyle,
    pub sans: SansStyle,
    pub nabla_upright: bool,
    pub partial_upright: bool,
    /// `mathrm=text` etc.: the command selects the text font (fontspec
    /// family) instead of a Unicode math alphabet. All default to text.
    pub mathrm_text: bool,
    pub mathit_text: bool,
    pub mathbf_text: bool,
    pub mathsf_text: bool,
    pub mathtt_text: bool,
}

impl Default for MathOptions {
    /// `\unimathsetup{math-style=TeX}` and `mathrm=text,...,mathtt=text`.
    fn default() -> Self {
        MathOptions {
            normal: NormalStyle::Tex,
            bold: BoldStyle::Tex,
            sans: SansStyle::Upright,
            nabla_upright: true,
            partial_upright: false,
            mathrm_text: true,
            mathit_text: true,
            mathbf_text: true,
            mathsf_text: true,
            mathtt_text: true,
        }
    }
}

impl MathOptions {
    /// Applies option keys in order (package options, `\setmathfont[...]`,
    /// `\unimathsetup{...}`). Unknown keys are returned.
    pub fn apply(&mut self, keys: &[KeyVal]) -> Vec<KeyVal> {
        let mut unknown = Vec::new();
        for kv in keys {
            let v = kv.value.as_deref().unwrap_or("");
            let text = |v: &str| match v {
                "text" => Some(true),
                "sym" => Some(false),
                _ => None,
            };
            let ok = match kv.key.as_str() {
                "math-style" => match v {
                    "ISO" => {
                        *self = MathOptions {
                            normal: NormalStyle::Iso,
                            bold: BoldStyle::Iso,
                            sans: SansStyle::Italic,
                            nabla_upright: true,
                            partial_upright: false,
                            ..*self
                        };
                        true
                    }
                    "TeX" => {
                        *self = MathOptions {
                            normal: NormalStyle::Tex,
                            bold: BoldStyle::Tex,
                            sans: SansStyle::Upright,
                            nabla_upright: true,
                            partial_upright: false,
                            ..*self
                        };
                        true
                    }
                    "french" => {
                        *self = MathOptions {
                            normal: NormalStyle::French,
                            bold: BoldStyle::Upright,
                            sans: SansStyle::Upright,
                            nabla_upright: true,
                            partial_upright: true,
                            ..*self
                        };
                        true
                    }
                    "upright" => {
                        *self = MathOptions {
                            normal: NormalStyle::Upright,
                            bold: BoldStyle::Upright,
                            sans: SansStyle::Upright,
                            nabla_upright: true,
                            partial_upright: true,
                            ..*self
                        };
                        true
                    }
                    "literal" => {
                        *self = MathOptions {
                            normal: NormalStyle::Literal,
                            bold: BoldStyle::Literal,
                            sans: SansStyle::Literal,
                            ..*self
                        };
                        true
                    }
                    _ => false,
                },
                "normal-style" => {
                    self.normal = match v {
                        "ISO" => NormalStyle::Iso,
                        "TeX" => NormalStyle::Tex,
                        "french" => NormalStyle::French,
                        "upright" => NormalStyle::Upright,
                        "literal" => NormalStyle::Literal,
                        _ => {
                            unknown.push(kv.clone());
                            continue;
                        }
                    };
                    true
                }
                "bold-style" => {
                    self.bold = match v {
                        "ISO" => BoldStyle::Iso,
                        "TeX" => BoldStyle::Tex,
                        "upright" => BoldStyle::Upright,
                        "literal" => BoldStyle::Literal,
                        _ => {
                            unknown.push(kv.clone());
                            continue;
                        }
                    };
                    true
                }
                "sans-style" => {
                    self.sans = match v {
                        "italic" => SansStyle::Italic,
                        "upright" => SansStyle::Upright,
                        "literal" => SansStyle::Literal,
                        _ => {
                            unknown.push(kv.clone());
                            continue;
                        }
                    };
                    true
                }
                "nabla" => {
                    self.nabla_upright = v != "italic";
                    true
                }
                "partial" => {
                    self.partial_upright = v == "upright";
                    true
                }
                "mathrm" => text(v).map(|t| self.mathrm_text = t).is_some(),
                "mathit" => text(v).map(|t| self.mathit_text = t).is_some(),
                "mathbf" => text(v).map(|t| self.mathbf_text = t).is_some(),
                "mathsf" => text(v).map(|t| self.mathsf_text = t).is_some(),
                "mathtt" => text(v).map(|t| self.mathtt_text = t).is_some(),
                // Font-loading keys that do not affect alphabets.
                "range" | "version" | "Scale" | "script-features" | "sscript-features"
                | "script-font" | "sscript-font" | "colon" | "slash-delimiter" | "active-frac"
                | "vargreek-shape" | "trace" | "warnings-off" => true,
                _ => false,
            };
            if !ok {
                unknown.push(kv.clone());
            }
        }
        unknown
    }

    /// Options from a whole document: `\usepackage[...]{unicode-math}`,
    /// `\setmathfont[...]{..}[...]` and `\unimathsetup{...}` in order.
    pub fn from_source(src: &str) -> MathOptions {
        let code = keyval::blank_comments(src);
        let mut o = MathOptions::default();
        let mut i = 0;
        while let Some(p) = code[i..].find('\\') {
            let start = i + p;
            let rest = &code[start + 1..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphabetic())
                .collect();
            let after = start + 1 + name.len();
            let mut a = keyval::ArgScanner::new(&code, after);
            match name.as_str() {
                "usepackage" => {
                    let opts = a.optional();
                    if a.mandatory()
                        .map(|p| p.split(',').any(|x| x.trim() == "unicode-math"))
                        == Some(true)
                        && let Some(opts) = opts
                    {
                        o.apply(&keyval::parse(opts));
                    }
                }
                "setmathfont" => {
                    if let Some(x) = a.optional() {
                        o.apply(&keyval::parse(x));
                    }
                    let _ = a.mandatory();
                    if let Some(x) = a.optional() {
                        o.apply(&keyval::parse(x));
                    }
                }
                "unimathsetup" => {
                    if let Some(x) = a.mandatory() {
                        o.apply(&keyval::parse(x));
                    }
                }
                _ => {}
            }
            i = after.max(start + 1);
        }
        o
    }
}

/// What an alphabet command does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphabetCommand {
    /// Selects a Unicode math alphabet.
    Symbol(Alphabet),
    /// `\symbf`/`\symsf`/`\symbfsf`: alphabet depends on bold-style/sans-style.
    StyledBold,
    StyledSans,
    StyledBoldSans,
    /// Switches to the text font (`mathrm=text`...): `series_bold`, `shape_italic`, `family`.
    TextFont {
        bold: bool,
        italic: bool,
        family: TextFamily,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextFamily {
    Roman,
    Sans,
    Mono,
}

/// Resolves an alphabet command name (with or without backslash).
pub fn command(name: &str, o: &MathOptions) -> Option<AlphabetCommand> {
    use Alphabet::*;
    use AlphabetCommand::*;
    let n = name.trim_start_matches('\\');
    Some(match n {
        "mathrm" if o.mathrm_text => TextFont {
            bold: false,
            italic: false,
            family: TextFamily::Roman,
        },
        "mathit" if o.mathit_text => TextFont {
            bold: false,
            italic: true,
            family: TextFamily::Roman,
        },
        "mathbf" if o.mathbf_text => TextFont {
            bold: true,
            italic: false,
            family: TextFamily::Roman,
        },
        "mathsf" if o.mathsf_text => TextFont {
            bold: false,
            italic: false,
            family: TextFamily::Sans,
        },
        "mathtt" if o.mathtt_text => TextFont {
            bold: false,
            italic: false,
            family: TextFamily::Mono,
        },
        "mathrm" | "symup" | "mathup" | "symrm" => Symbol(Up),
        "mathit" | "symit" => Symbol(It),
        "mathbf" | "symbf" => StyledBold,
        "mathsf" | "symsf" => StyledSans,
        "mathtt" | "symtt" => Symbol(Tt),
        "symbfup" | "mathbfup" => Symbol(Bf),
        "symbfit" | "mathbfit" => Symbol(BfIt),
        "symsfup" | "mathsfup" => Symbol(Sf),
        "symsfit" | "mathsfit" => Symbol(SfIt),
        "symbfsf" | "mathbfsf" => StyledBoldSans,
        "symbfsfup" | "mathbfsfup" => Symbol(BfSf),
        "symbfsfit" | "mathbfsfit" => Symbol(BfSfIt),
        "symbb" | "mathbb" => Symbol(Bb),
        "symbbit" | "mathbbit" => Symbol(BbIt),
        "symcal" | "mathcal" | "symscr" | "mathscr" => Symbol(Scr),
        "symbfcal" | "mathbfcal" | "symbfscr" | "mathbfscr" => Symbol(BfScr),
        "symfrak" | "mathfrak" => Symbol(Frak),
        "symbffrak" | "mathbffrak" => Symbol(BfFrak),
        _ => return None,
    })
}

/// Greek and related symbol commands → base code points.
pub fn symbol_command(name: &str) -> Option<char> {
    let n = name.trim_start_matches('\\');
    Some(match n {
        "alpha" => 'α',
        "beta" => 'β',
        "gamma" => 'γ',
        "delta" => 'δ',
        "epsilon" => 'ϵ',
        "varepsilon" => 'ε',
        "zeta" => 'ζ',
        "eta" => 'η',
        "theta" => 'θ',
        "vartheta" => 'ϑ',
        "iota" => 'ι',
        "kappa" => 'κ',
        "varkappa" => 'ϰ',
        "lambda" => 'λ',
        "mu" => 'μ',
        "nu" => 'ν',
        "xi" => 'ξ',
        "omicron" => 'ο',
        "pi" => 'π',
        "varpi" => 'ϖ',
        "rho" => 'ρ',
        "varrho" => 'ϱ',
        "sigma" => 'σ',
        "varsigma" => 'ς',
        "tau" => 'τ',
        "upsilon" => 'υ',
        "phi" => 'ϕ',
        "varphi" => 'φ',
        "chi" => 'χ',
        "psi" => 'ψ',
        "omega" => 'ω',
        "Alpha" => 'Α',
        "Beta" => 'Β',
        "Gamma" => 'Γ',
        "Delta" => 'Δ',
        "Epsilon" => 'Ε',
        "Zeta" => 'Ζ',
        "Eta" => 'Η',
        "Theta" => 'Θ',
        "Iota" => 'Ι',
        "Kappa" => 'Κ',
        "Lambda" => 'Λ',
        "Mu" => 'Μ',
        "Nu" => 'Ν',
        "Xi" => 'Ξ',
        "Omicron" => 'Ο',
        "Pi" => 'Π',
        "Rho" => 'Ρ',
        "Sigma" => 'Σ',
        "Tau" => 'Τ',
        "Upsilon" => 'Υ',
        "Phi" => 'Φ',
        "Chi" => 'Χ',
        "Psi" => 'Ψ',
        "Omega" => 'Ω',
        "varTheta" => 'ϴ',
        "partial" => '∂',
        "nabla" => '∇',
        "imath" => 'ı',
        "jmath" => 'ȷ',
        _ => return None,
    })
}

const LATIN_BASE: &[(Alphabet, u32)] = &[
    (Alphabet::Bf, 0x1D400),
    (Alphabet::It, 0x1D434),
    (Alphabet::BfIt, 0x1D468),
    (Alphabet::Scr, 0x1D49C),
    (Alphabet::BfScr, 0x1D4D0),
    (Alphabet::Frak, 0x1D504),
    (Alphabet::Bb, 0x1D538),
    (Alphabet::BfFrak, 0x1D56C),
    (Alphabet::Sf, 0x1D5A0),
    (Alphabet::BfSf, 0x1D5D4),
    (Alphabet::SfIt, 0x1D608),
    (Alphabet::BfSfIt, 0x1D63C),
    (Alphabet::Tt, 0x1D670),
];

/// Letterlike-symbols holes in the Mathematical Alphanumeric Symbols block.
fn latin_hole(a: Alphabet, c: char) -> Option<char> {
    use Alphabet::*;
    Some(match (a, c) {
        (It, 'h') => 'ℎ',
        (Scr, 'B') => 'ℬ',
        (Scr, 'E') => 'ℰ',
        (Scr, 'F') => 'ℱ',
        (Scr, 'H') => 'ℋ',
        (Scr, 'I') => 'ℐ',
        (Scr, 'L') => 'ℒ',
        (Scr, 'M') => 'ℳ',
        (Scr, 'R') => 'ℛ',
        (Scr, 'e') => 'ℯ',
        (Scr, 'g') => 'ℊ',
        (Scr, 'o') => 'ℴ',
        (Frak, 'C') => 'ℭ',
        (Frak, 'H') => 'ℌ',
        (Frak, 'I') => 'ℑ',
        (Frak, 'R') => 'ℜ',
        (Frak, 'Z') => 'ℨ',
        (Bb, 'C') => 'ℂ',
        (Bb, 'H') => 'ℍ',
        (Bb, 'N') => 'ℕ',
        (Bb, 'P') => 'ℙ',
        (Bb, 'Q') => 'ℚ',
        (Bb, 'R') => 'ℝ',
        (Bb, 'Z') => 'ℤ',
        (BbIt, 'D') => 'ⅅ',
        (BbIt, 'd') => 'ⅆ',
        (BbIt, 'e') => 'ⅇ',
        (BbIt, 'i') => 'ⅈ',
        (BbIt, 'j') => 'ⅉ',
        _ => return None,
    })
}

/// Greek block order: Α..Ω with ϴ in the U+03A2 slot, ∇, α..ω, ∂ ϵ ϑ ϰ ϕ ϱ ϖ.
fn greek_index(c: char) -> Option<u32> {
    let u = c as u32;
    Some(match c {
        'ϴ' => 17,
        '∇' => 25,
        '∂' => 51,
        'ϵ' => 52,
        'ϑ' => 53,
        'ϰ' => 54,
        'ϕ' => 55,
        'ϱ' => 56,
        'ϖ' => 57,
        _ if (0x391..=0x3A9).contains(&u) && u != 0x3A2 => u - 0x391,
        _ if (0x3B1..=0x3C9).contains(&u) => u - 0x3B1 + 26,
        _ => return None,
    })
}

const GREEK_BASE: &[(Alphabet, u32)] = &[
    (Alphabet::Bf, 0x1D6A8),
    (Alphabet::It, 0x1D6E2),
    (Alphabet::BfIt, 0x1D71C),
    (Alphabet::BfSf, 0x1D756),
    (Alphabet::BfSfIt, 0x1D790),
];

const DIGIT_BASE: &[(Alphabet, u32)] = &[
    (Alphabet::Bf, 0x1D7CE),
    (Alphabet::Bb, 0x1D7D8),
    (Alphabet::Sf, 0x1D7E2),
    (Alphabet::BfSf, 0x1D7EC),
    (Alphabet::Tt, 0x1D7F6),
];

/// The code point `c` takes in alphabet `a` (unchanged when the alphabet
/// has no such character, as unicode-math does).
pub fn map_char(a: Alphabet, c: char) -> char {
    if a == Alphabet::Up {
        return c;
    }
    if c.is_ascii_alphabetic() {
        if let Some(h) = latin_hole(a, c) {
            return h;
        }
        let idx = if c.is_ascii_uppercase() {
            c as u32 - 'A' as u32
        } else {
            c as u32 - 'a' as u32 + 26
        };
        if let Some((_, base)) = LATIN_BASE.iter().find(|(x, _)| *x == a) {
            return char::from_u32(base + idx).unwrap_or(c);
        }
        return c;
    }
    if c.is_ascii_digit() {
        if let Some((_, base)) = DIGIT_BASE.iter().find(|(x, _)| *x == a) {
            return char::from_u32(base + (c as u32 - '0' as u32)).unwrap_or(c);
        }
        return c;
    }
    if a == Alphabet::It {
        match c {
            'ı' => return '\u{1D6A4}',
            'ȷ' => return '\u{1D6A5}',
            _ => {}
        }
    }
    if let Some(idx) = greek_index(c)
        && let Some((_, base)) = GREEK_BASE.iter().find(|(x, _)| *x == a)
    {
        return char::from_u32(base + idx).unwrap_or(c);
    }
    c
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    LatinUpper,
    LatinLower,
    GreekUpper,
    GreekLower,
    Nabla,
    Partial,
    Other,
}

fn class(c: char) -> Class {
    match c {
        'A'..='Z' => Class::LatinUpper,
        'a'..='z' | 'ı' | 'ȷ' => Class::LatinLower,
        '∇' => Class::Nabla,
        '∂' => Class::Partial,
        _ => match greek_index(c) {
            Some(i) if i < 25 => Class::GreekUpper,
            Some(_) => Class::GreekLower,
            None => Class::Other,
        },
    }
}

/// Alphabet of a letter typed directly in math (no alphabet command).
pub fn normal_alphabet(c: char, o: &MathOptions) -> Alphabet {
    use Alphabet::*;
    let upright = match (class(c), o.normal) {
        (_, NormalStyle::Literal) => true,
        (Class::Nabla, _) => o.nabla_upright,
        (Class::Partial, _) => o.partial_upright,
        (Class::Other, _) => true,
        (_, NormalStyle::Upright) => true,
        (_, NormalStyle::Iso) => false,
        (Class::GreekUpper, NormalStyle::Tex) => true,
        (_, NormalStyle::Tex) => false,
        (Class::LatinUpper | Class::GreekUpper | Class::GreekLower, NormalStyle::French) => true,
        (_, NormalStyle::French) => false,
    };
    if upright { Up } else { It }
}

fn bold_alphabet(c: char, o: &MathOptions) -> Alphabet {
    use Alphabet::*;
    let upright = match (class(c), o.bold) {
        (_, BoldStyle::Literal | BoldStyle::Upright) => true,
        (Class::Other, _) => true,
        (_, BoldStyle::Iso) => false,
        (Class::GreekLower | Class::Partial, BoldStyle::Tex) => false,
        (_, BoldStyle::Tex) => true,
    };
    if upright { Bf } else { BfIt }
}

/// Maps `c` under an optional alphabet command. Returns `None` for text-font
/// commands (the character is set in the text font, unchanged).
pub fn map_in(cmd: Option<AlphabetCommand>, c: char, o: &MathOptions) -> Option<char> {
    let a = match cmd {
        None => normal_alphabet(c, o),
        Some(AlphabetCommand::Symbol(a)) => a,
        Some(AlphabetCommand::StyledBold) => bold_alphabet(c, o),
        Some(AlphabetCommand::StyledSans) => {
            if o.sans == SansStyle::Italic {
                Alphabet::SfIt
            } else {
                Alphabet::Sf
            }
        }
        Some(AlphabetCommand::StyledBoldSans) => {
            if o.sans == SansStyle::Italic {
                Alphabet::BfSfIt
            } else {
                Alphabet::BfSf
            }
        }
        Some(AlphabetCommand::TextFont { .. }) => return None,
    };
    Some(map_char(a, c))
}

/// Maps a simple math expression (letters, digits, Greek commands and
/// `\cmd{...}` alphabet groups; spaces ignored) to the characters unicode-math
/// typesets. Anything else is copied. Returns the string and whether any part
/// went to the text font.
pub fn map_expression(expr: &str, o: &MathOptions) -> String {
    let mut out = String::new();
    map_into(expr, None, o, &mut out);
    out
}

fn map_into(expr: &str, cmd: Option<AlphabetCommand>, o: &MathOptions, out: &mut String) {
    let mut i = 0;
    while i < expr.len() {
        let rest = &expr[i..];
        let c = rest.chars().next().unwrap();
        if c == '\\' {
            let name: String = rest[1..]
                .chars()
                .take_while(|c| c.is_ascii_alphabetic())
                .collect();
            let mut next = i + 1 + name.len();
            if let Some(ac) = command(&name, o) {
                let mut a = keyval::ArgScanner::new(expr, next);
                if let Some(arg) = a.mandatory() {
                    map_into(arg, Some(ac), o, out);
                    next = a.pos;
                }
            } else if let Some(sym) = symbol_command(&name) {
                out.push(map_in(cmd, sym, o).unwrap_or(sym));
            } else {
                out.push('\\');
                out.push_str(&name);
            }
            i = next;
            continue;
        }
        if !c.is_whitespace() && c != '{' && c != '}' {
            out.push(map_in(cmd, c, o).unwrap_or(c));
        }
        i += c.len_utf8();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tex_style_defaults() {
        let o = MathOptions::default();
        assert_eq!(map_expression(r"h x A", &o), "ℎ𝑥𝐴");
        assert_eq!(map_expression(r"\alpha\beta\Gamma\Delta", &o), "𝛼𝛽ΓΔ");
        assert_eq!(
            map_expression(r"\mathbb{RZN}\mathcal{ABL}\mathfrak{gA}", &o),
            "ℝℤℕ𝒜ℬℒ𝔤𝔄"
        );
        assert_eq!(
            map_expression(
                r"\symbf{xA}\symbfit{vA}\symsf{Ab}\symtt{ab}\symbfsf{Ab}",
                &o
            ),
            "𝐱𝐀𝒗𝑨𝖠𝖻𝚊𝚋𝗔𝗯"
        );
        assert_eq!(map_expression(r"\mathbf{xA}\mathit{hA}", &o), "xAhA");
        assert_eq!(
            map_expression(r"\symbf{\alpha\Gamma}\partial\nabla\mathbb{1}\symbf{2}", &o),
            "𝜶𝚪𝜕∇𝟙𝟐"
        );
    }

    #[test]
    fn iso_and_french() {
        let mut o = MathOptions::default();
        o.apply(&keyval::parse("math-style=ISO"));
        assert_eq!(
            map_expression(r"\Gamma\alpha A\symbf{A\alpha}", &o),
            "𝛤𝛼𝐴𝑨𝜶"
        );
        let mut f = MathOptions::default();
        f.apply(&keyval::parse("math-style=french"));
        assert_eq!(map_expression(r"xA\alpha\Gamma\partial", &f), "𝑥AαΓ∂");
    }

    #[test]
    fn options_from_source() {
        let o = MathOptions::from_source(
            "\\usepackage[mathbf=sym]{unicode-math}\n\\setmathfont{STIX Two Math}[bold-style=ISO]",
        );
        assert!(!o.mathbf_text);
        assert_eq!(o.bold, BoldStyle::Iso);
        assert_eq!(map_expression(r"\mathbf{x}", &o), "𝒙");
    }
}
