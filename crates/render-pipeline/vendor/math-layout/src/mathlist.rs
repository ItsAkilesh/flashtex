//! The math list model: atoms with a class, a nucleus, and optional scripts.

/// TeX's eight atom classes (TeXbook ch. 17).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AtomClass {
    Ord,
    Op,
    Bin,
    Rel,
    Open,
    Close,
    Punct,
    Inner,
}

impl AtomClass {
    pub(crate) fn index(self) -> usize {
        match self {
            AtomClass::Ord => 0,
            AtomClass::Op => 1,
            AtomClass::Bin => 2,
            AtomClass::Rel => 3,
            AtomClass::Open => 4,
            AtomClass::Close => 5,
            AtomClass::Punct => 6,
            AtomClass::Inner => 7,
        }
    }
}

/// Limit placement for `Op` atoms (`\limits`, `\nolimits`, `\displaylimits`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Limits {
    /// Limits in display style, scripts otherwise (TeX's default).
    #[default]
    DisplayLimits,
    Limits,
    NoLimits,
}

/// What sits in the nucleus of an atom.
#[derive(Debug, Clone, PartialEq)]
pub enum Nucleus {
    /// A single symbol resolved to a glyph by the metrics provider.
    Symbol(char),
    /// A braced subformula.
    List(MathList),
    /// `\frac{num}{den}`. `thickness` overrides the default rule thickness
    /// (0 gives `\atop`-style stacking).
    Fraction {
        numerator: MathList,
        denominator: MathList,
        thickness: Option<f64>,
    },
    /// `\sqrt{radicand}` or `\sqrt[degree]{radicand}`.
    Radical {
        radicand: MathList,
        degree: Option<MathList>,
    },
    /// Upright operator text such as `\lim` or `\sin` (`\operator@font`):
    /// each character is a text glyph of the roman font, no italic correction.
    Text(String),
    /// `\overline{body}`: body under a rule (Rule 9).
    Overline(MathList),
    /// `\underline{body}`: body over a rule (Rule 10).
    Underline(MathList),
    /// `{\displaystyle body}` and friends: an explicit style override for the
    /// body, boxed as an ordinary atom. This is the `\mathchoice`-free way to
    /// force a style; the body's own sub-formulas derive from it as usual.
    Styled {
        style: crate::style::Style,
        body: MathList,
    },
    /// `\hat{base}` and friends; `accent` is the accent symbol.
    Accent { accent: char, base: MathList },
    /// `\left l body \right r`. `None` is a null delimiter.
    Delimited {
        left: Option<char>,
        right: Option<char>,
        body: MathList,
    },
    /// `{}`: an empty ordinary atom.
    Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Atom {
    pub class: AtomClass,
    pub nucleus: Nucleus,
    pub superscript: Option<MathList>,
    pub subscript: Option<MathList>,
    pub limits: Limits,
}

impl Atom {
    pub fn new(class: AtomClass, nucleus: Nucleus) -> Atom {
        Atom {
            class,
            nucleus,
            superscript: None,
            subscript: None,
            limits: Limits::default(),
        }
    }

    /// A symbol atom whose class comes from the default classification table.
    pub fn symbol(ch: char) -> Atom {
        let (class, limits) = default_class(ch);
        Atom {
            limits,
            ..Atom::new(class, Nucleus::Symbol(ch))
        }
    }

    pub fn ord(ch: char) -> Atom {
        Atom::new(AtomClass::Ord, Nucleus::Symbol(ch))
    }

    pub fn op(ch: char) -> Atom {
        Atom::new(AtomClass::Op, Nucleus::Symbol(ch))
    }

    pub fn bin(ch: char) -> Atom {
        Atom::new(AtomClass::Bin, Nucleus::Symbol(ch))
    }

    pub fn rel(ch: char) -> Atom {
        Atom::new(AtomClass::Rel, Nucleus::Symbol(ch))
    }

    pub fn open(ch: char) -> Atom {
        Atom::new(AtomClass::Open, Nucleus::Symbol(ch))
    }

    pub fn close(ch: char) -> Atom {
        Atom::new(AtomClass::Close, Nucleus::Symbol(ch))
    }

    pub fn punct(ch: char) -> Atom {
        Atom::new(AtomClass::Punct, Nucleus::Symbol(ch))
    }

    pub fn group(list: MathList) -> Atom {
        Atom::new(AtomClass::Ord, Nucleus::List(list))
    }

    pub fn frac(numerator: MathList, denominator: MathList) -> Atom {
        Atom::new(
            AtomClass::Inner,
            Nucleus::Fraction {
                numerator,
                denominator,
                thickness: None,
            },
        )
    }

    pub fn sqrt(radicand: MathList) -> Atom {
        Atom::new(
            AtomClass::Ord,
            Nucleus::Radical {
                radicand,
                degree: None,
            },
        )
    }

    /// `\sqrt[degree]{radicand}`.
    pub fn root(degree: MathList, radicand: MathList) -> Atom {
        Atom::new(
            AtomClass::Ord,
            Nucleus::Radical {
                radicand,
                degree: Some(degree),
            },
        )
    }

    /// `\lim`, `\sin`, …: an `Op` atom whose nucleus is upright text.
    /// `\lim`-style operators take limits in display style by default; pass
    /// `Limits::NoLimits` (as LaTeX does for `\sin`) with [`Atom::with_limits`].
    pub fn text_op(text: &str) -> Atom {
        Atom::new(AtomClass::Op, Nucleus::Text(text.to_string()))
    }

    pub fn overline(body: MathList) -> Atom {
        Atom::new(AtomClass::Ord, Nucleus::Overline(body))
    }

    pub fn underline(body: MathList) -> Atom {
        Atom::new(AtomClass::Ord, Nucleus::Underline(body))
    }

    /// `{\displaystyle body}` etc.
    pub fn styled(style: crate::style::Style, body: MathList) -> Atom {
        Atom::new(AtomClass::Ord, Nucleus::Styled { style, body })
    }

    pub fn accent(accent: char, base: MathList) -> Atom {
        Atom::new(AtomClass::Ord, Nucleus::Accent { accent, base })
    }

    pub fn left_right(left: Option<char>, right: Option<char>, body: MathList) -> Atom {
        Atom::new(AtomClass::Inner, Nucleus::Delimited { left, right, body })
    }

    pub fn with_sup(mut self, sup: MathList) -> Atom {
        self.superscript = Some(sup);
        self
    }

    pub fn with_sub(mut self, sub: MathList) -> Atom {
        self.subscript = Some(sub);
        self
    }

    pub fn with_limits(mut self, limits: Limits) -> Atom {
        self.limits = limits;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MathList {
    pub atoms: Vec<Atom>,
}

impl MathList {
    pub fn new(atoms: Vec<Atom>) -> MathList {
        MathList { atoms }
    }

    /// Builds a list from symbols with the default classification.
    pub fn symbols(text: &str) -> MathList {
        MathList {
            atoms: text.chars().map(Atom::symbol).collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.atoms.is_empty()
    }
}

impl From<Atom> for MathList {
    fn from(atom: Atom) -> MathList {
        MathList { atoms: vec![atom] }
    }
}

/// Default class and limit behaviour of a symbol, after plain.tex's
/// `\mathcode`/`\mathchardef` assignments for the symbols this crate knows.
pub fn default_class(ch: char) -> (AtomClass, Limits) {
    use AtomClass::*;
    let class = match ch {
        '+' | '-' | '\u{2212}' | '\u{22C5}' | '\u{00D7}' | '\u{00F7}' | '\u{00B1}' | '\u{2213}'
        | '\u{2217}' | '\u{2218}' | '\u{2229}' | '\u{222A}' | '\u{2228}' | '\u{2227}'
        | '\u{2295}' | '\u{2297}' | '\u{2216}' => Bin,
        '=' | '<' | '>' | ':' | '\u{2264}' | '\u{2265}' | '\u{2261}' | '\u{2248}' | '\u{2260}'
        | '\u{223C}' | '\u{2282}' | '\u{2283}' | '\u{2286}' | '\u{2287}' | '\u{2208}'
        | '\u{220B}' | '\u{2190}' | '\u{2192}' | '\u{2194}' | '\u{21D0}' | '\u{21D2}'
        | '\u{21D4}' | '\u{2225}' | '\u{22A5}' | '\u{2223}' => Rel,
        '(' | '[' | '{' | '\u{27E8}' | '\u{2308}' | '\u{230A}' => Open,
        ')' | ']' | '}' | '\u{27E9}' | '\u{2309}' | '\u{230B}' => Close,
        ',' | ';' => Punct,
        '\u{2211}' | '\u{220F}' | '\u{2210}' | '\u{222B}' | '\u{222E}' | '\u{22C2}'
        | '\u{22C3}' | '\u{2A01}' | '\u{2A02}' | '\u{2A00}' | '\u{22C1}' | '\u{22C0}' => Op,
        _ => Ord,
    };
    // plain.tex: \int and \oint are \intop\nolimits.
    let limits = match ch {
        '\u{222B}' | '\u{222E}' => Limits::NoLimits,
        _ => Limits::DisplayLimits,
    };
    (class, limits)
}
