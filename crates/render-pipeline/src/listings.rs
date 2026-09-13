//! The listings package (listings.sty/lstmisc.sty 1.11b, lstlang1.sty) as
//! pdfLaTeX sets it: the keys of `lstlisting`, `\lstinline`,
//! `\lstinputlisting`, `\lstset` and `\lstdefinestyle`; the C, C++, Java
//! and Python definitions; and the output algorithm that turns a line of
//! code into boxes and kerns (`\lst@OutputToken`, `\lst@FillFixed`,
//! `\lst@CalcLostSpaceAndOutput`, `\lst@ProcessSpace`,
//! `\lst@ProcessTabulator`).
//!
//! Everything here is source text and arithmetic; font measurements come
//! through [`Measure`], implemented by the typesetter.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

/// The font a piece of a listing is set in: family, series, shape and the
/// size declaration (hundredths of a point; 0 is the size in force).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct FontState {
    pub mono: bool,
    pub bold: bool,
    pub italic: bool,
    pub size_cpt: u16,
}

/// One font declaration of a style key (`basicstyle=\ttfamily\small`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Decl {
    Mono(bool),
    Bold(bool),
    Italic(bool),
    NormalFont,
    Size(&'static str),
}

const SIZE_NAMES: [&str; 10] = ["tiny", "scriptsize", "footnotesize", "small", "normalsize", "large", "Large", "LARGE", "huge", "Huge"];

/// A style key's value: its font declarations in order.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Style {
    decls: Vec<Decl>,
}

impl Style {
    /// The font declarations of `tex`; colours, `\color{..}` and other
    /// macros are skipped (they move nothing).
    pub fn parse(tex: &str) -> Style {
        let bytes = tex.as_bytes();
        let mut decls = Vec::new();
        let mut i = 0usize;
        while i < bytes.len() {
            if bytes[i] != b'\\' {
                i += 1;
                continue;
            }
            let start = i + 1;
            let mut j = start;
            while j < bytes.len() && bytes[j].is_ascii_alphabetic() {
                j += 1;
            }
            let name = &tex[start..j];
            i = j.max(start + 1);
            let decl = match name {
                "ttfamily" | "tt" | "texttt" => Some(Decl::Mono(true)),
                "rmfamily" | "rm" | "sffamily" | "sf" | "textrm" | "textsf" => Some(Decl::Mono(false)),
                "bfseries" | "bf" | "textbf" => Some(Decl::Bold(true)),
                "mdseries" | "textmd" => Some(Decl::Bold(false)),
                "itshape" | "it" | "slshape" | "sl" | "em" | "textit" | "textsl" | "emph" => Some(Decl::Italic(true)),
                "upshape" | "textup" => Some(Decl::Italic(false)),
                "normalfont" | "textnormal" => Some(Decl::NormalFont),
                "color" | "textcolor" | "colorbox" => {
                    // `\color[<model>]{<spec>}`: skip the arguments.
                    i = skip_arguments(tex, i, 1);
                    None
                }
                other => SIZE_NAMES.iter().find(|s| **s == other).map(|s| Decl::Size(s)),
            };
            decls.extend(decl);
        }
        Style { decls }
    }

    /// `font` with this style's declarations applied under the class size.
    pub fn apply(&self, mut font: FontState, class_size: u32) -> FontState {
        for d in &self.decls {
            match d {
                Decl::Mono(m) => font.mono = *m,
                Decl::Bold(b) => font.bold = *b,
                Decl::Italic(it) => font.italic = *it,
                Decl::NormalFont => {
                    font.mono = false;
                    font.bold = false;
                    font.italic = false;
                }
                Decl::Size(name) => font.size_cpt = crate::verbatim::size_of(name, class_size).unwrap_or(0),
            }
        }
        font
    }

}

/// Skips `[..]` and `count` `{..}` groups after `at`.
fn skip_arguments(tex: &str, mut at: usize, count: usize) -> usize {
    let bytes = tex.as_bytes();
    let blank = |at: &mut usize| {
        while *at < bytes.len() && bytes[*at].is_ascii_whitespace() {
            *at += 1;
        }
    };
    blank(&mut at);
    if bytes.get(at) == Some(&b'[') {
        at = tex[at..].find(']').map_or(bytes.len(), |k| at + k + 1);
    }
    for _ in 0..count {
        blank(&mut at);
        if bytes.get(at) != Some(&b'{') {
            break;
        }
        at = matching(tex, at).map_or(bytes.len(), |k| k + 1);
    }
    at
}

/// The index of the `}` matching the `{` at `open`.
fn matching(tex: &str, open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (i, b) in tex.bytes().enumerate().skip(open) {
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// A TeX dimension as written: its value and unit (font units resolved
/// against the font current where listings evaluates it).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dimen {
    pub value: f64,
    pub unit: Unit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Pt,
    Em,
    Ex,
}

impl Dimen {
    pub const fn pt(value: f64) -> Dimen {
        Dimen { value, unit: Unit::Pt }
    }

    pub const fn em(value: f64) -> Dimen {
        Dimen { value, unit: Unit::Em }
    }

    /// Points, with `em`/`ex` the font's `\fontdimen6`/`\fontdimen5`.
    pub fn resolve(self, quad: f64, x_height: f64) -> f64 {
        match self.unit {
            Unit::Pt => self.value,
            Unit::Em => self.value * quad,
            Unit::Ex => self.value * x_height,
        }
    }

    /// `12pt`, `-0.5em`, `1in`, `3mm`, `\z@`; `None` for anything else.
    pub fn parse(s: &str) -> Option<Dimen> {
        let s = s.trim();
        if s == "\\z@" || s == "0" {
            return Some(Dimen::pt(0.0));
        }
        let split = s.find(|c: char| c.is_ascii_alphabetic()).unwrap_or(s.len());
        let value: f64 = s[..split].trim().parse().ok()?;
        let (unit, factor) = match s[split..].trim() {
            "pt" => (Unit::Pt, 1.0),
            "em" => (Unit::Em, 1.0),
            "ex" => (Unit::Ex, 1.0),
            "bp" => (Unit::Pt, 72.27 / 72.0),
            "in" => (Unit::Pt, 72.27),
            "cm" => (Unit::Pt, 72.27 / 2.54),
            "mm" => (Unit::Pt, 72.27 / 25.4),
            "pc" => (Unit::Pt, 12.0),
            "dd" => (Unit::Pt, 1238.0 / 1157.0),
            "cc" => (Unit::Pt, 12.0 * 1238.0 / 1157.0),
            "sp" => (Unit::Pt, 1.0 / 65536.0),
            _ => return None,
        };
        Some(Dimen { value: value * factor, unit })
    }
}

/// A skip: natural width with stretch and shrink, in points (listings'
/// `aboveskip`/`belowskip` are evaluated before `basicstyle`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkipSpec {
    pub natural: Dimen,
    pub stretch: f64,
    pub shrink: f64,
}

impl SkipSpec {
    /// `\medskipamount` (latex.ltx: 6pt plus 2pt minus 2pt).
    pub const MEDSKIP: SkipSpec = SkipSpec {
        natural: Dimen::pt(6.0),
        stretch: 2.0,
        shrink: 2.0,
    };

    pub fn parse(s: &str) -> Option<SkipSpec> {
        let s = s.trim();
        match s {
            "\\medskipamount" => return Some(SkipSpec::MEDSKIP),
            "\\smallskipamount" => return Some(SkipSpec { natural: Dimen::pt(3.0), stretch: 1.0, shrink: 1.0 }),
            "\\bigskipamount" => return Some(SkipSpec { natural: Dimen::pt(12.0), stretch: 4.0, shrink: 4.0 }),
            _ => {}
        }
        let (natural, rest) = match s.find(" plus").or_else(|| s.find(" minus")) {
            Some(k) => (&s[..k], &s[k..]),
            None => (s, ""),
        };
        let natural = Dimen::parse(natural)?;
        let part = |word: &str| -> f64 {
            rest.split_once(word)
                .and_then(|(_, r)| Dimen::parse(r.trim().split_whitespace().next().unwrap_or("")))
                .map_or(0.0, |d| d.value)
        };
        Some(SkipSpec { natural, stretch: part("plus"), shrink: part("minus") })
    }
}

/// `columns=[<pos>]<kind>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Columns {
    Fixed,
    Flexible,
    FullFlexible,
    SpaceFlexible,
}

/// The output position of `columns=[l|c|r]` (`\lst@outputpos`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Align {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Numbers {
    None,
    Left,
    Right,
}

/// `frame=`: which sides are drawn (`trbl`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Frame {
    pub top: bool,
    pub right: bool,
    pub bottom: bool,
    pub left: bool,
}

impl Frame {
    pub fn any(self) -> bool {
        self.top || self.right || self.bottom || self.left
    }
}

/// A string delimiter (`morestring=[b]"` / `[d]` / `[s]{open}{close}`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StringDelim {
    pub open: &'static str,
    pub close: &'static str,
    /// `[b]`: a backslash before the closing delimiter escapes it.
    pub backslash: bool,
}

/// A language definition (lstlang1.sty).
#[derive(Debug, PartialEq, Eq)]
pub struct Language {
    pub name: &'static str,
    pub keywords: &'static [&'static str],
    /// `morekeywords=[2]{..}`.
    pub keywords2: &'static [&'static str],
    pub sensitive: bool,
    pub line_comments: &'static [&'static str],
    pub block_comments: &'static [(&'static str, &'static str)],
    pub strings: &'static [StringDelim],
    /// `moredelim=*[directive]\#` with `moredirectives`.
    pub directives: &'static [&'static str],
}

const C_KEYWORDS: &[&str] = &[
    "auto", "break", "case", "char", "const", "continue", "default", "do", "double", "else", "enum", "extern", "float", "for", "goto", "if", "int", "long",
    "register", "return", "short", "signed", "sizeof", "static", "struct", "switch", "typedef", "union", "unsigned", "void", "volatile", "while",
];
const CPP_KEYWORDS: &[&str] = &[
    "auto", "break", "case", "char", "const", "continue", "default", "do", "double", "else", "enum", "extern", "float", "for", "goto", "if", "int", "long",
    "register", "return", "short", "signed", "sizeof", "static", "struct", "switch", "typedef", "union", "unsigned", "void", "volatile", "while", "and",
    "and_eq", "asm", "bad_cast", "bad_typeid", "bitand", "bitor", "bool", "catch", "class", "compl", "const_cast", "delete", "dynamic_cast", "explicit",
    "export", "false", "friend", "inline", "mutable", "namespace", "new", "not", "not_eq", "operator", "or", "or_eq", "private", "protected", "public",
    "reinterpret_cast", "static_cast", "template", "this", "throw", "true", "try", "typeid", "type_info", "typename", "using", "virtual", "wchar_t", "xor",
    "xor_eq",
];
const C_DIRECTIVES: &[&str] = &["define", "elif", "else", "endif", "error", "if", "ifdef", "ifndef", "line", "include", "pragma", "undef", "warning"];
const C_STRINGS: &[StringDelim] = &[
    StringDelim { open: "\"", close: "\"", backslash: true },
    StringDelim { open: "'", close: "'", backslash: true },
];
const JAVA_KEYWORDS: &[&str] = &[
    "abstract", "boolean", "break", "byte", "case", "catch", "char", "class", "const", "continue", "default", "do", "double", "else", "extends", "false",
    "final", "finally", "float", "for", "goto", "if", "implements", "import", "instanceof", "int", "interface", "label", "long", "native", "new", "null",
    "package", "private", "protected", "public", "return", "short", "static", "super", "switch", "synchronized", "this", "throw", "throws", "transient",
    "true", "try", "void", "volatile", "while",
];
const PYTHON2_KEYWORDS: &[&str] = &[
    "and", "as", "assert", "break", "class", "continue", "def", "del", "elif", "else", "except", "exec", "finally", "for", "from", "global", "if", "import",
    "in", "is", "lambda", "not", "or", "pass", "print", "raise", "return", "try", "while", "with", "yield",
];
const PYTHON2_BUILTINS: &[&str] = &[
    "abs", "all", "any", "basestring", "bin", "bool", "bytearray", "callable", "chr", "classmethod", "cmp", "compile", "complex", "delattr", "dict", "dir",
    "divmod", "enumerate", "eval", "execfile", "file", "filter", "float", "format", "frozenset", "getattr", "globals", "hasattr", "hash", "help", "hex",
    "id", "input", "int", "isinstance", "issubclass", "iter", "len", "list", "locals", "long", "map", "max", "memoryview", "min", "next", "object", "oct",
    "open", "ord", "pow", "property", "range", "raw_input", "reduce", "reload", "repr", "reversed", "round", "set", "setattr", "slice", "sorted",
    "staticmethod", "str", "sum", "super", "tuple", "type", "unichr", "unicode", "vars", "xrange", "zip", "__import__", "apply", "buffer", "coerce", "intern",
];
const PYTHON3_KEYWORDS: &[&str] = &[
    "and", "as", "assert", "break", "class", "continue", "def", "del", "elif", "else", "except", "finally", "for", "from", "global", "if", "import", "in",
    "is", "lambda", "not", "or", "pass", "raise", "return", "try", "while", "with", "yield", "False", "None", "True", "async", "await", "nonlocal", "match",
    "case",
];
const PYTHON3_BUILTINS: &[&str] = &[
    "abs", "all", "any", "bin", "bool", "bytearray", "callable", "chr", "classmethod", "compile", "complex", "delattr", "dict", "dir", "divmod",
    "enumerate", "eval", "filter", "float", "format", "frozenset", "getattr", "globals", "hasattr", "hash", "help", "hex", "id", "input", "int",
    "isinstance", "issubclass", "iter", "len", "list", "locals", "map", "max", "memoryview", "min", "next", "object", "oct", "open", "ord", "pow",
    "property", "range", "repr", "reversed", "round", "set", "setattr", "slice", "sorted", "staticmethod", "str", "sum", "super", "tuple", "type", "vars",
    "zip", "__import__", "aiter", "anext", "ascii", "breakpoint", "bytes", "exec",
];
const PYTHON_STRINGS: &[StringDelim] = &[
    StringDelim { open: "'''", close: "'''", backslash: false },
    StringDelim { open: "\"\"\"", close: "\"\"\"", backslash: false },
    StringDelim { open: "'", close: "'", backslash: true },
    StringDelim { open: "\"", close: "\"", backslash: true },
];

/// `[ANSI]C` (the `defaultdialect` of listings.cfg).
pub static C: Language = Language {
    name: "C",
    keywords: C_KEYWORDS,
    keywords2: &[],
    sensitive: true,
    line_comments: &["//"],
    block_comments: &[("/*", "*/")],
    strings: C_STRINGS,
    directives: C_DIRECTIVES,
};
/// `[ISO]C++` (based on `[ANSI]C`).
pub static CPP: Language = Language {
    name: "C++",
    keywords: CPP_KEYWORDS,
    keywords2: &[],
    sensitive: true,
    line_comments: &["//"],
    block_comments: &[("/*", "*/")],
    strings: C_STRINGS,
    directives: C_DIRECTIVES,
};
pub static JAVA: Language = Language {
    name: "Java",
    keywords: JAVA_KEYWORDS,
    keywords2: &[],
    sensitive: true,
    line_comments: &["//"],
    block_comments: &[("/*", "*/")],
    strings: C_STRINGS,
    directives: &[],
};
/// `[2]Python` (listings.cfg `defaultdialect=[2]Python`).
pub static PYTHON2: Language = Language {
    name: "Python",
    keywords: PYTHON2_KEYWORDS,
    keywords2: PYTHON2_BUILTINS,
    sensitive: true,
    line_comments: &["#"],
    block_comments: &[],
    strings: PYTHON_STRINGS,
    directives: &[],
};
pub static PYTHON3: Language = Language {
    name: "Python",
    keywords: PYTHON3_KEYWORDS,
    keywords2: PYTHON3_BUILTINS,
    sensitive: true,
    line_comments: &["#"],
    block_comments: &[],
    strings: PYTHON_STRINGS,
    directives: &[],
};

/// The language `language=[<dialect>]<name>` names (case-insensitive, as
/// listings compares them), `None` when it is not defined here.
pub fn language(value: &str) -> Option<&'static Language> {
    let v = strip_braces(value.trim());
    let (dialect, name) = match v.strip_prefix('[') {
        Some(rest) => match rest.split_once(']') {
            Some((d, n)) => (d.trim().to_ascii_lowercase(), n.trim().to_ascii_lowercase()),
            None => (String::new(), v.to_ascii_lowercase()),
        },
        None => (String::new(), v.to_ascii_lowercase()),
    };
    match (name.as_str(), dialect.as_str()) {
        ("c", "" | "ansi") => Some(&C),
        ("c++", "" | "iso") => Some(&CPP),
        ("java", "") => Some(&JAVA),
        ("python", "" | "2") => Some(&PYTHON2),
        ("python", "3") => Some(&PYTHON3),
        _ => None,
    }
}

fn strip_braces(s: &str) -> &str {
    let t = s.trim();
    if t.starts_with('{') && matching(t, 0) == Some(t.len() - 1) {
        t[1..t.len() - 1].trim()
    } else {
        t
    }
}

/// Every listings key this module lays out, with the values in force.
#[derive(Debug, Clone, PartialEq)]
pub struct Options {
    pub basicstyle: Style,
    /// Class 1..3 keyword styles; an unset class uses class 1's
    /// (`\lst@ProvideStyle`).
    pub keywordstyles: [Option<Style>; 3],
    pub commentstyle: Style,
    pub stringstyle: Style,
    pub identifierstyle: Style,
    pub directivestyle: Option<Style>,
    /// `columns` for the fixed and the flexible class, and which is used
    /// (`flexiblecolumns`).
    pub fixed_columns: Columns,
    pub flexible_columns: Columns,
    pub flexible: bool,
    pub align: Align,
    pub basewidth: (Dimen, Dimen),
    pub numbers: Numbers,
    pub numberstyle: Style,
    pub numbersep: Dimen,
    pub stepnumber: i64,
    pub firstnumber: Option<i64>,
    pub numberblanklines: bool,
    pub frame: Frame,
    pub framesep: Dimen,
    pub framerule: Dimen,
    pub rulesep: Dimen,
    pub xleftmargin: Dimen,
    pub xrightmargin: Dimen,
    pub aboveskip: SkipSpec,
    pub belowskip: SkipSpec,
    pub breaklines: bool,
    pub breakindent: Dimen,
    pub breakatwhitespace: bool,
    pub breakautoindent: bool,
    pub tabsize: usize,
    pub gobble: usize,
    pub showspaces: bool,
    pub showstringspaces: bool,
    pub keepspaces: bool,
    pub language: Option<&'static Language>,
    pub morekeywords: Vec<(usize, String)>,
    pub deletekeywords: Vec<String>,
    pub sensitive: Option<bool>,
    pub firstline: Option<usize>,
    pub lastline: Option<usize>,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            basicstyle: Style::default(),
            keywordstyles: [Some(Style::parse("\\bfseries")), None, None],
            // listings.sty's `EmptyStyle` hook: `\let\lst@commentstyle\itshape`.
            commentstyle: Style::parse("\\itshape"),
            stringstyle: Style::default(),
            identifierstyle: Style::default(),
            directivestyle: None,
            fixed_columns: Columns::Fixed,
            flexible_columns: Columns::Flexible,
            flexible: false,
            align: Align::Center,
            basewidth: (Dimen::em(0.6), Dimen::em(0.45)),
            numbers: Numbers::None,
            numberstyle: Style::default(),
            numbersep: Dimen::pt(10.0),
            stepnumber: 1,
            firstnumber: None,
            numberblanklines: true,
            frame: Frame::default(),
            framesep: Dimen::pt(3.0),
            framerule: Dimen::pt(0.4),
            rulesep: Dimen::pt(2.0),
            xleftmargin: Dimen::pt(0.0),
            xrightmargin: Dimen::pt(0.0),
            aboveskip: SkipSpec::MEDSKIP,
            belowskip: SkipSpec::MEDSKIP,
            breaklines: false,
            breakindent: Dimen::pt(20.0),
            breakatwhitespace: false,
            breakautoindent: true,
            tabsize: 8,
            gobble: 0,
            showspaces: false,
            showstringspaces: true,
            keepspaces: false,
            language: None,
            morekeywords: Vec::new(),
            deletekeywords: Vec::new(),
            sensitive: None,
            firstline: None,
            lastline: None,
        }
    }
}

/// `true`/`false`/empty (a bare key means true).
fn boolean(value: Option<&str>) -> bool {
    !matches!(value.map(str::trim), Some("false" | "f" | "no"))
}

/// Splits a key-value list at top-level commas into `(key, value)`.
pub fn key_values(list: &str) -> Vec<(&str, Option<&str>)> {
    let mut out = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    let bytes = list.as_bytes();
    for i in 0..=bytes.len() {
        match bytes.get(i) {
            Some(b'{') => depth += 1,
            Some(b'}') => depth = depth.saturating_sub(1),
            Some(b',') | None if depth == 0 => {
                let item = list[start..i].trim();
                if !item.is_empty() {
                    match item.split_once('=') {
                        Some((k, v)) => out.push((k.trim(), Some(strip_braces(v)))),
                        None => out.push((item, None)),
                    }
                }
                start = i + 1;
            }
            _ => {}
        }
    }
    out
}

impl Options {
    /// Applies a key-value list; `styles` holds the `\lstdefinestyle`s.
    pub fn apply(&mut self, list: &str, styles: &BTreeMap<String, String>) {
        self.apply_depth(list, styles, 0);
    }

    fn apply_depth(&mut self, list: &str, styles: &BTreeMap<String, String>, depth: usize) {
        for (key, value) in key_values(list) {
            let v = value.unwrap_or("");
            let dimen = |d: &mut Dimen| {
                if let Some(x) = Dimen::parse(v) {
                    *d = x;
                }
            };
            match key {
                "style" if depth < 8 => {
                    if let Some(s) = styles.get(v.trim()).cloned() {
                        self.apply_depth(&s, styles, depth + 1);
                    }
                }
                "language" => {
                    self.language = language(v);
                    self.morekeywords.clear();
                    self.deletekeywords.clear();
                }
                "basicstyle" => self.basicstyle = Style::parse(v),
                "keywordstyle" => {
                    let (class, style) = class_argument(v);
                    if (1..=3).contains(&class) {
                        self.keywordstyles[class - 1] = Some(Style::parse(style));
                    }
                }
                "commentstyle" => self.commentstyle = Style::parse(v),
                "stringstyle" => self.stringstyle = Style::parse(v),
                "identifierstyle" => self.identifierstyle = Style::parse(v),
                "directivestyle" => self.directivestyle = Some(Style::parse(v)),
                "columns" => {
                    let (pos, kind) = match v.trim().strip_prefix('[') {
                        Some(rest) => rest.split_once(']').map_or(("", v), |(p, k)| (p.trim(), k.trim())),
                        None => ("", v.trim()),
                    };
                    match pos {
                        "l" => self.align = Align::Left,
                        "c" => self.align = Align::Center,
                        "r" => self.align = Align::Right,
                        _ => {}
                    }
                    match kind {
                        "fixed" => {
                            self.fixed_columns = Columns::Fixed;
                            self.flexible = false;
                        }
                        "flexible" => {
                            self.flexible_columns = Columns::Flexible;
                            self.flexible = true;
                        }
                        "fullflexible" => {
                            self.flexible_columns = Columns::FullFlexible;
                            self.flexible = true;
                        }
                        "spaceflexible" => {
                            self.flexible_columns = Columns::SpaceFlexible;
                            self.flexible = true;
                        }
                        _ => {}
                    }
                }
                "flexiblecolumns" => self.flexible = boolean(value),
                "basewidth" => {
                    let parts: Vec<&str> = v.split(',').collect();
                    if let Some(f) = parts.first().and_then(|p| Dimen::parse(p)) {
                        self.basewidth.0 = f;
                        self.basewidth.1 = parts.get(1).and_then(|p| Dimen::parse(p)).unwrap_or(f);
                    }
                }
                "numbers" => {
                    self.numbers = match v.trim() {
                        "left" => Numbers::Left,
                        "right" => Numbers::Right,
                        _ => Numbers::None,
                    }
                }
                "numberstyle" => self.numberstyle = Style::parse(v),
                "numbersep" => dimen(&mut self.numbersep),
                "stepnumber" => self.stepnumber = v.trim().parse().unwrap_or(1),
                "firstnumber" => self.firstnumber = v.trim().parse().ok(),
                "numberblanklines" => self.numberblanklines = boolean(value),
                "frame" => {
                    let sides = match v.trim() {
                        "none" => "",
                        "leftline" => "l",
                        "topline" => "t",
                        "bottomline" => "b",
                        "lines" => "tb",
                        "single" | "shadowbox" => "trbl",
                        other => other,
                    };
                    let has = |c: char| sides.contains(c) || sides.contains(c.to_ascii_uppercase());
                    self.frame = Frame { top: has('t'), right: has('r'), bottom: has('b'), left: has('l') };
                }
                "framesep" => dimen(&mut self.framesep),
                "framerule" => dimen(&mut self.framerule),
                "rulesep" => dimen(&mut self.rulesep),
                "xleftmargin" => dimen(&mut self.xleftmargin),
                "xrightmargin" => dimen(&mut self.xrightmargin),
                "aboveskip" => {
                    if let Some(s) = SkipSpec::parse(v) {
                        self.aboveskip = s;
                    }
                }
                "belowskip" => {
                    if let Some(s) = SkipSpec::parse(v) {
                        self.belowskip = s;
                    }
                }
                "breaklines" => self.breaklines = boolean(value),
                "breakindent" => dimen(&mut self.breakindent),
                "breakatwhitespace" => self.breakatwhitespace = boolean(value),
                "breakautoindent" => self.breakautoindent = boolean(value),
                "tabsize" => self.tabsize = v.trim().parse().ok().filter(|t| *t > 0).unwrap_or(self.tabsize),
                "gobble" => self.gobble = v.trim().parse().unwrap_or(0),
                "showspaces" => self.showspaces = boolean(value),
                "showstringspaces" => self.showstringspaces = boolean(value),
                "keepspaces" => self.keepspaces = boolean(value),
                "morekeywords" | "keywords" => {
                    let (class, words) = class_argument(v);
                    if key == "keywords" {
                        self.morekeywords.retain(|(c, _)| *c != class);
                    }
                    self.morekeywords.extend(words.split(',').map(str::trim).filter(|w| !w.is_empty()).map(|w| (class, w.to_string())));
                }
                "deletekeywords" => {
                    let (_, words) = class_argument(v);
                    self.deletekeywords.extend(words.split(',').map(str::trim).filter(|w| !w.is_empty()).map(str::to_string));
                }
                "sensitive" => self.sensitive = Some(boolean(value)),
                "firstline" => self.firstline = v.trim().parse().ok(),
                "lastline" => self.lastline = v.trim().parse().ok(),
                _ => {}
            }
        }
    }

    /// The keyword class of an identifier (1-based), if it is one.
    pub fn keyword_class(&self, word: &str) -> Option<usize> {
        let sensitive = self.sensitive.unwrap_or(self.language.is_some_and(|l| l.sensitive));
        let eq = |a: &str| if sensitive { a == word } else { a.eq_ignore_ascii_case(word) };
        if self.deletekeywords.iter().any(|w| eq(w)) {
            return None;
        }
        if let Some((class, _)) = self.morekeywords.iter().find(|(_, w)| eq(w)) {
            return Some(*class);
        }
        let lang = self.language?;
        if lang.keywords.iter().any(|w| eq(w)) {
            Some(1)
        } else if lang.keywords2.iter().any(|w| eq(w)) {
            Some(2)
        } else {
            None
        }
    }

    /// The style of keyword class `class` (1-based).
    pub fn keyword_style(&self, class: usize) -> &Style {
        self.keywordstyles
            .get(class.saturating_sub(1))
            .and_then(Option::as_ref)
            .or(self.keywordstyles[0].as_ref())
            .expect("class 1 always has a style")
    }

    /// The columns in use.
    pub fn columns(&self) -> Columns {
        if self.flexible {
            self.flexible_columns
        } else {
            self.fixed_columns
        }
    }
}

/// `[<class>]<value>` of `keywordstyle`/`morekeywords`.
fn class_argument(v: &str) -> (usize, &str) {
    match v.trim().strip_prefix('[') {
        Some(rest) => match rest.split_once(']') {
            Some((c, value)) => (c.trim().parse().unwrap_or(1), strip_braces(value)),
            None => (1, v),
        },
        None => (1, v),
    }
}

/// The options of a listing at byte `at` of `source`: the defaults, every
/// `\lstset` before `at` in source order (`\lstdefinestyle`s recorded for
/// `style=`), then `own` (the environment's or command's own keys).
pub fn options_at(source: &str, at: usize, own: Option<&str>) -> Options {
    let mut options = Options::default();
    let mut styles: BTreeMap<String, String> = BTreeMap::new();
    for event in settings(source).iter().take_while(|e| e.at < at) {
        match &event.style {
            Some(name) => {
                styles.insert(name.clone(), event.keys.clone());
            }
            None => options.apply(&event.keys, &styles),
        }
    }
    if let Some(own) = own {
        options.apply(own, &styles);
    }
    options
}

/// One `\lstset{keys}` (`style` `None`) or `\lstdefinestyle{name}{keys}` of
/// a document, at the byte where its command starts.
#[derive(Debug)]
struct Setting {
    at: usize,
    style: Option<String>,
    keys: String,
}

thread_local! {
    /// The last document scanned: (text pointer, length) and its settings.
    static SETTINGS: RefCell<Option<(usize, usize, Rc<Vec<Setting>>)>> = const { RefCell::new(None) };
}

/// The listings settings of `source` in order, scanned once per document.
fn settings(source: &str) -> Rc<Vec<Setting>> {
    let key = (source.as_ptr() as usize, source.len());
    SETTINGS.with(|cell| {
        let mut slot = cell.borrow_mut();
        match &*slot {
            Some((p, l, s)) if (*p, *l) == key => s.clone(),
            _ => {
                let s = Rc::new(scan_settings(source));
                *slot = Some((key.0, key.1, s.clone()));
                s
            }
        }
    })
}

fn scan_settings(source: &str) -> Vec<Setting> {
    let mut out = Vec::new();
    let bytes = source.as_bytes();
    let end = source.len();
    let mut i = 0usize;
    while i < end {
        match bytes[i] {
            b'%' if i == 0 || bytes[i - 1] != b'\\' => {
                while i < end && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'\\' => {
                let command = i;
                let name_end = source[i + 1..].find(|c: char| !c.is_ascii_alphabetic()).map_or(source.len(), |k| i + 1 + k);
                let name = &source[i + 1..name_end];
                i = name_end.max(i + 1);
                match name {
                    "lstset" => {
                        if let Some((body, after)) = group_after(source, i) {
                            out.push(Setting { at: command, style: None, keys: body.to_string() });
                            i = after;
                        }
                    }
                    "lstdefinestyle" => {
                        if let Some((style_name, after)) = group_after(source, i) {
                            if let Some((body, after)) = group_after(source, after) {
                                out.push(Setting { at: command, style: Some(style_name.trim().to_string()), keys: body.to_string() });
                                i = after;
                            }
                        }
                    }
                    "begin" => {
                        // Skip verbatim bodies, whose text is not TeX.
                        if let Some((env, after)) = group_after(source, i) {
                            if matches!(env.trim(), "verbatim" | "verbatim*" | "lstlisting" | "comment") {
                                let tag = format!("\\end{{{}}}", env.trim());
                                i = source[after..].find(&tag).map_or(source.len(), |k| after + k + tag.len());
                            } else {
                                i = after;
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => i += 1,
        }
    }
    out
}

/// The `{..}` group right after `at` (blanks first) and the byte after it.
fn group_after(source: &str, at: usize) -> Option<(&str, usize)> {
    let rest = source.get(at..)?;
    let trimmed = rest.trim_start();
    let open = at + (rest.len() - trimmed.len());
    if !trimmed.starts_with('{') {
        return None;
    }
    let close = matching(source, open)?;
    Some((&source[open + 1..close], close + 1))
}

/// The `[..]` right after `at` (blanks first), if any, and the byte after it.
pub fn bracket_after(source: &str, at: usize) -> Option<(&str, usize)> {
    let rest = source.get(at..)?;
    let trimmed = rest.trim_start_matches([' ', '\t']);
    let open = at + (rest.len() - trimmed.len());
    if !trimmed.starts_with('[') {
        return None;
    }
    let mut depth = 0usize;
    for (k, b) in source.bytes().enumerate().skip(open + 1) {
        match b {
            b'{' => depth += 1,
            b'}' => depth = depth.saturating_sub(1),
            b']' if depth == 0 => return Some((&source[open + 1..k], k + 1)),
            b'\n' if depth == 0 => return None,
            _ => {}
        }
    }
    None
}

/// Font measurements listings needs, supplied by the typesetter.
pub trait Measure {
    /// `\fontdimen6` (quad) and `\fontdimen5` (x-height) of `font`, in pt.
    fn quad_ex(&mut self, font: FontState) -> (f64, f64);
    /// The width of each character of `text` set literally in `font`.
    fn char_widths(&mut self, font: FontState, text: &str) -> Vec<f64>;
    /// `\fontdimen2` of `font` (the width of `\ `), in pt.
    fn space(&mut self, font: FontState) -> f64;
}

/// Which part of the language a token belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Class {
    Plain,
    Identifier,
    Keyword(usize),
    Directive,
    Comment,
    String,
}

/// One output item of a line, left to right.
#[derive(Debug, Clone, PartialEq)]
pub enum Piece {
    /// `\lst@Kern`: lost space inserted before a token.
    Kern(f64),
    /// An output box: its characters (with source byte offsets relative to
    /// the line's text) at `offsets` inside a box `width` wide.
    Token {
        text: String,
        bytes: Vec<(usize, usize)>,
        font: FontState,
        class: Class,
        width: f64,
        offsets: Vec<f64>,
    },
    /// A legal break after the previous box (`breaklines`).
    Break,
}

/// One output line of a listing.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OutLine {
    pub pieces: Vec<Piece>,
    /// The printed line number, when numbers are shown on this line.
    pub number: Option<i64>,
    /// `\lst@lostspace` at the start of the line's first output (the
    /// automatic break indent of `breakautoindent`).
    pub leading: f64,
}

/// latex.ltx `\verbatim@nolig@list`: listings makes each of these append
/// `\lst@nolig` (`\leavevmode\kern\z@`, no column of its own) to the token
/// being built before the character itself (`\lst@do@noligs`).
const NOLIG: [char; 6] = ['`', '<', '>', ',', '\'', '-'];

/// One element of the token being built: a character, or `\lst@nolig`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Elem {
    Char(char, usize, usize),
    NoLig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    None,
    LineComment,
    BlockComment(&'static str),
    String(StringDelim),
}

/// The listings state machine over one listing (modes persist across lines).
struct Engine<'o, 'm> {
    opts: &'o Options,
    measure: &'m mut dyn Measure,
    class_size: u32,
    base: FontState,
    basic: FontState,
    /// `\lst@width`: the column width, from the basic font.
    width: f64,
    mode: Mode,
    // Per line.
    pieces: Vec<Piece>,
    lostspace: f64,
    pos: i64,
    at_bol: bool,
    newline: bool,
    whitespace: bool,
    letter: bool,
    token: Vec<Elem>,
    length: usize,
    lastother: Option<char>,
    leading: Option<f64>,
    inline: bool,
    /// `#` opened a C preprocessor directive on this line.
    directive_pending: bool,
}

impl Engine<'_, '_> {
    fn font_of(&self, class: Class) -> FontState {
        let o = self.opts;
        let basic = self.basic;
        let style = match class {
            Class::Plain => return basic,
            Class::Identifier => &o.identifierstyle,
            Class::Keyword(k) => o.keyword_style(k),
            Class::Directive => o.directivestyle.as_ref().unwrap_or(o.keyword_style(1)),
            Class::Comment => &o.commentstyle,
            Class::String => &o.stringstyle,
        };
        style.apply(basic, self.class_size)
    }

    fn mode_class(&self) -> Option<Class> {
        match self.mode {
            Mode::None => None,
            Mode::LineComment | Mode::BlockComment(_) => Some(Class::Comment),
            Mode::String(_) => Some(Class::String),
        }
    }

    fn keep_spaces(&self) -> bool {
        self.opts.showspaces || self.opts.keepspaces || (matches!(self.mode, Mode::String(_)) && self.opts.showstringspaces)
    }

    fn visible_space(&self) -> bool {
        self.opts.showspaces || (matches!(self.mode, Mode::String(_)) && self.opts.showstringspaces)
    }

    fn append(&mut self, c: char, start: usize, end: usize) {
        self.length += 1;
        self.token.push(Elem::Char(c, start, end));
    }

    /// The characters of the token being built.
    fn token_chars(&self) -> impl Iterator<Item = (char, usize, usize)> + '_ {
        self.token.iter().filter_map(|e| match e {
            Elem::Char(c, s, e) => Some((*c, *s, *e)),
            Elem::NoLig => None,
        })
    }

    fn append_letter(&mut self, c: char, start: usize, end: usize) {
        if !self.letter {
            self.output_other();
            self.letter = true;
        }
        self.append(c, start, end);
    }

    fn append_other(&mut self, c: char, start: usize, end: usize) {
        if self.letter {
            self.output();
            self.letter = false;
        }
        self.lastother = Some(c);
        self.append(c, start, end);
    }

    /// `\lst@Output`: an identifier token.
    fn output(&mut self) {
        if self.length > 0 {
            let class = match self.mode_class() {
                Some(c) => c,
                None => {
                    let word: String = self.token_chars().map(|t| t.0).collect();
                    match self.opts.keyword_class(&word) {
                        Some(k) => Class::Keyword(k),
                        None if self.directive_pending => {
                            if self.opts.language.is_some_and(|l| l.directives.contains(&word.as_str())) {
                                Class::Directive
                            } else {
                                Class::Identifier
                            }
                        }
                        None => Class::Identifier,
                    }
                }
            };
            self.directive_pending = false;
            self.output_token(class);
        }
        self.lastother = None;
    }

    /// `\lst@OutputOther`.
    fn output_other(&mut self) {
        if self.length > 0 {
            let class = self.mode_class().unwrap_or(Class::Plain);
            self.output_token(class);
        }
    }

    /// `\lst@PrintToken`.
    fn print_token(&mut self) {
        if self.letter {
            self.output();
            self.letter = false;
        } else {
            self.output_other();
            self.lastother = None;
        }
    }

    /// `\lst@OutputToken` with `\lst@CalcLostSpaceAndOutput`.
    fn output_token(&mut self, class: Class) {
        // `\lst@TrackNewLines` / `\lst@NewLine`.
        if self.at_bol {
            self.at_bol = false;
            self.newline = true;
            self.leading = Some(self.lostspace.max(0.0));
        }
        let columns = self.opts.columns();
        // `\lst@OutputLostSpace`.
        let use_lost = match columns {
            Columns::Fixed | Columns::Flexible => true,
            Columns::FullFlexible => self.newline,
            Columns::SpaceFlexible => self.whitespace || self.newline,
        };
        if use_lost && self.lostspace > 0.0 {
            self.pieces.push(Piece::Kern(self.lostspace));
            self.lostspace = 0.0;
        }
        let font = self.font_of(class);
        let text: String = self.token_chars().map(|t| t.0).collect();
        let bytes: Vec<(usize, usize)> = self.token_chars().map(|t| (t.1, t.2)).collect();
        let glyphs: String = text.chars().filter(|c| *c != ' ').collect();
        let widths = self.measure.char_widths(font, &glyphs);
        let natural: f64 = widths.iter().sum::<f64>() + text.chars().filter(|c| *c == ' ').count() as f64 * self.measure.space(font);
        let n = self.length as f64;
        let (width, offsets) = match columns {
            Columns::Fixed => {
                let boxw = n * self.width;
                let (left, right) = match self.opts.align {
                    Align::Center => (true, true),
                    Align::Left => (false, true),
                    Align::Right => (true, false),
                };
                // `\lst@FillFixed`: `\lst@hss` before every element but the
                // first (`\lst@nolig` counts), plus the outer `\hss`es.
                let elements = self.token.len();
                let glues = elements.saturating_sub(1) + usize::from(left) + usize::from(right);
                let g = if glues == 0 { 0.0 } else { (boxw - natural) / glues as f64 };
                let mut x = if left { g } else { 0.0 };
                let mut offsets = Vec::new();
                let mut wi = widths.iter();
                let space = self.measure.space(font);
                for (k, e) in self.token.iter().enumerate() {
                    if k > 0 {
                        x += g;
                    }
                    if let Elem::Char(c, _, _) = e {
                        offsets.push(x);
                        x += if *c == ' ' { space } else { *wi.next().unwrap_or(&0.0) };
                    }
                }
                (boxw, offsets)
            }
            _ => {
                let mut x = 0.0;
                let mut wi = widths.iter();
                let space = self.measure.space(font);
                let offsets = text
                    .chars()
                    .map(|c| {
                        let at = x;
                        x += if c == ' ' { space } else { *wi.next().unwrap_or(&0.0) };
                        at
                    })
                    .collect();
                (natural, offsets)
            }
        };
        self.lostspace += n * self.width - width;
        self.pos -= self.length as i64;
        let (left_insert, right_insert) = match (columns, self.opts.align) {
            (Columns::FullFlexible | Columns::SpaceFlexible, _) => (false, false),
            (_, Align::Center) => (true, true),
            (_, Align::Right) => (true, false),
            (_, Align::Left) => (false, true),
        };
        if self.lostspace > 0.0 && left_insert {
            if self.opts.align == Align::Center {
                self.lostspace *= 0.5;
            }
            self.pieces.push(Piece::Kern(self.lostspace));
            if self.opts.align != Align::Center {
                self.lostspace = 0.0;
            }
        }
        self.pieces.push(Piece::Token { text, bytes, font, class, width, offsets });
        if self.lostspace > 0.0 && right_insert {
            self.pieces.push(Piece::Kern(self.lostspace));
            self.lostspace = 0.0;
        }
        // `\lst@PostOutput`: `\lst@discretionary`.
        if self.opts.breaklines && (!self.opts.breakatwhitespace || self.whitespace) {
            self.pieces.push(Piece::Break);
        }
        self.newline = false;
        self.token.clear();
        self.length = 0;
    }

    /// `\lst@ProcessSpace`.
    fn space(&mut self, start: usize, end: usize) {
        let out = if self.visible_space() { '\u{2423}' } else { ' ' };
        if self.keep_spaces() {
            self.print_token();
            self.whitespace = true;
            self.append_other(out, start, end);
            self.print_token();
        } else if !self.at_bol || self.length > 0 {
            // `\lst@AppendSpecialSpace`.
            if self.whitespace {
                self.print_token();
                self.lostspace += self.width;
                self.pos -= 1;
            } else {
                self.print_token();
                self.whitespace = true;
                self.append_other(out, start, end);
                self.print_token();
            }
        } else {
            self.lostspace += self.width;
            self.pos -= 1;
            self.whitespace = true;
        }
    }

    /// `\lst@ProcessTabulator`.
    fn tab(&mut self, start: usize, end: usize) {
        self.print_token();
        self.whitespace = true;
        let tab = self.opts.tabsize as i64;
        while self.pos < 1 {
            self.pos += tab;
        }
        let length = self.pos as usize;
        if self.keep_spaces() {
            let out = if self.visible_space() { '\u{2423}' } else { ' ' };
            for _ in 0..length {
                self.append_other(out, start, end);
            }
            self.output_other();
        } else if !self.at_bol {
            // `\lst@GotoTabStop`: an empty box as wide as `\ ` plus the rest
            // as lost space.
            let space = self.measure.space(self.basic);
            self.lostspace += length as f64 * self.width - space;
            self.pieces.push(Piece::Kern(space));
            self.pos -= length as i64;
        } else {
            self.lostspace += length as f64 * self.width;
        }
        self.length = 0;
        self.token.clear();
        self.pos = 0;
    }
}

impl Engine<'_, '_> {
    // A `#` directive is pending after `#` at the start of a C line.
    #[allow(clippy::too_many_lines)]
    fn line(&mut self, text: &str) -> OutLine {
        self.pieces.clear();
        self.lostspace = 0.0;
        self.pos = 0;
        self.at_bol = true;
        self.newline = false;
        self.whitespace = true;
        self.letter = false;
        self.token.clear();
        self.length = 0;
        self.lastother = None;
        self.leading = None;
        self.directive_pending = false;
        let lang = self.opts.language;
        let chars: Vec<(usize, char)> = text.char_indices().collect();
        let mut k = 0usize;
        while k < chars.len() {
            let (at, c) = chars[k];
            let rest = &text[at..];
            let end = at + c.len_utf8();
            // Delimiters (`\lst@DelimOpen`/`\lst@DelimClose`).
            match self.mode {
                Mode::None => {
                    if let Some(l) = lang {
                        if let Some(open) = l.line_comments.iter().find(|d| rest.starts_with(**d)) {
                            self.print_token();
                            self.mode = Mode::LineComment;
                            k += self.delimiter(&chars, k, open.len());
                            continue;
                        }
                        if let Some((open, close)) = l.block_comments.iter().find(|(o, _)| rest.starts_with(*o)) {
                            self.print_token();
                            self.mode = Mode::BlockComment(close);
                            k += self.delimiter(&chars, k, open.len());
                            continue;
                        }
                        if let Some(s) = l.strings.iter().find(|s| rest.starts_with(s.open)) {
                            self.print_token();
                            self.mode = Mode::String(*s);
                            k += self.delimiter(&chars, k, s.open.len());
                            continue;
                        }
                        if c == '#' && !l.directives.is_empty() && self.at_bol && self.length == 0 {
                            self.append_other(c, at, end);
                            self.print_token();
                            self.directive_pending = true;
                            k += 1;
                            continue;
                        }
                    }
                }
                Mode::BlockComment(close) if rest.starts_with(close) => {
                    self.print_token();
                    let n = self.delimiter(&chars, k, close.len());
                    self.mode = Mode::None;
                    k += n;
                    continue;
                }
                // `[b]`: `\lst@DefDelimBE` first outputs a pending identifier
                // (`\lst@Output` resets `\lst@lastother`), then a backslash
                // right before the delimiter escapes it.
                Mode::String(s) if rest.starts_with(s.close) && !(s.backslash && !self.letter && self.lastother == Some('\\')) => {
                    self.print_token();
                    let n = self.delimiter(&chars, k, s.close.len());
                    self.mode = Mode::None;
                    k += n;
                    continue;
                }
                Mode::String(s) if s.backslash && c == '\\' && self.lastother == Some('\\') => {
                    // `\\` inside a `[b]` string: the second backslash does
                    // not escape what follows.
                    self.append_other(c, at, end);
                    self.lastother = None;
                    k += 1;
                    continue;
                }
                _ => {}
            }
            match c {
                ' ' => self.space(at, end),
                '\t' => self.tab(at, end),
                '\u{c}' | '\r' => {}
                c if c.is_ascii_alphabetic() || matches!(c, '@' | '$' | '_') || !c.is_ascii() => {
                    self.whitespace = false;
                    self.append_letter(c, at, end);
                }
                c if c.is_ascii_digit() => {
                    self.whitespace = false;
                    if self.letter {
                        self.append_letter(c, at, end);
                    } else {
                        self.append_other(c, at, end);
                    }
                }
                c => {
                    self.whitespace = false;
                    if NOLIG.contains(&c) {
                        self.token.push(Elem::NoLig);
                    }
                    self.append_other(c, at, end);
                    if c == ')' && self.opts.breaklines {
                        self.output_other();
                    }
                }
            }
            k += 1;
        }
        // End of line: `\lst@XPrintToken`; a line comment ends.
        self.print_token();
        if self.mode == Mode::LineComment {
            self.mode = Mode::None;
        }
        OutLine {
            pieces: std::mem::take(&mut self.pieces),
            number: None,
            leading: self.leading.unwrap_or(0.0),
        }
    }

    /// Processes a delimiter of `len` bytes starting at char `k` as other
    /// characters in the new mode; returns how many chars it took.
    fn delimiter(&mut self, chars: &[(usize, char)], k: usize, len: usize) -> usize {
        let start = chars[k].0;
        let mut n = 0usize;
        while k + n < chars.len() && chars[k + n].0 < start + len {
            let (at, c) = chars[k + n];
            self.whitespace = false;
            if NOLIG.contains(&c) {
                self.token.push(Elem::NoLig);
            }
            self.append_other(c, at, at + c.len_utf8());
            n += 1;
        }
        self.print_token();
        n.max(1)
    }
}

/// `breaklines` (lstmisc.sty `\lst@discretionary` after every output box,
/// `\parshape` with `\lst@breakshape`, `\rightskip\@flushglue`): the
/// pieces of one line split at its [`Piece::Break`]s, each part as wide as
/// fits (`first_width` for the first part, `rest_width` after), with the
/// kerns after a break discarded as TeX discards them.
pub fn break_line(pieces: &[Piece], first_width: f64, rest_width: f64) -> Vec<Vec<Piece>> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut avail = first_width;
    loop {
        if !parts.is_empty() {
            while start < pieces.len() && matches!(pieces[start], Piece::Kern(_) | Piece::Break) {
                start += 1;
            }
        }
        let mut x = 0.0;
        let mut cut = None;
        let mut end = pieces.len();
        for (i, piece) in pieces.iter().enumerate().skip(start) {
            match piece {
                Piece::Break => cut = Some(i),
                Piece::Kern(k) => x += k,
                Piece::Token { width, .. } => {
                    if x + width > avail + 1e-6 {
                        if let Some(c) = cut {
                            end = c;
                            break;
                        }
                    }
                    x += width;
                }
            }
        }
        parts.push(pieces[start..end].to_vec());
        if end >= pieces.len() {
            return parts;
        }
        start = end + 1;
        avail = rest_width;
    }
}

/// The lines of a listing after `gobble`, `firstline`/`lastline`, with the
/// empty lines at the end dropped (`\lst@DeInit` discards the pending
/// `\lst@NewLine`s). Each entry is `(byte offset of the kept text within
/// the source line, kept text)`.
pub fn kept_lines<'a>(lines: &'a [&'a str], opts: &Options) -> Vec<(usize, usize, &'a str)> {
    let first = opts.firstline.unwrap_or(1).max(1);
    let last = opts.lastline.unwrap_or(usize::MAX);
    let mut out: Vec<(usize, usize, &str)> = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let number = index + 1;
        if number < first || number > last {
            continue;
        }
        let skip = line.char_indices().nth(opts.gobble).map_or(line.len(), |(b, _)| b);
        out.push((index, skip, &line[skip..]));
    }
    while out.last().is_some_and(|(_, _, t)| t.chars().all(|c| c == ' ' || c == '\t' || c == '\r')) {
        out.pop();
    }
    out
}

/// Lays out the kept lines of a listing: the pieces of every output line
/// and its line number. `base` is the font in force where the listing
/// starts (before `basicstyle`).
pub fn layout(lines: &[&str], opts: &Options, base: FontState, class_size: u32, inline: bool, measure: &mut dyn Measure) -> Vec<OutLine> {
    let basic = opts.basicstyle.apply(base, class_size);
    let (quad, x_height) = measure.quad_ex(basic);
    let basewidth = if opts.flexible { opts.basewidth.1 } else { opts.basewidth.0 };
    let width = basewidth.resolve(quad, x_height).max(0.0);
    let mut engine = Engine {
        opts,
        measure,
        class_size,
        base,
        basic,
        width,
        mode: Mode::None,
        pieces: Vec::new(),
        lostspace: 0.0,
        pos: 0,
        at_bol: true,
        newline: false,
        whitespace: true,
        letter: false,
        token: Vec::new(),
        length: 0,
        lastother: None,
        leading: None,
        inline,
        directive_pending: false,
    };
    let kept = kept_lines(lines, opts);
    let first_number = opts.firstnumber.unwrap_or_else(|| opts.firstline.unwrap_or(1) as i64);
    let step = opts.stepnumber.max(1);
    let mut out = Vec::with_capacity(kept.len());
    for (k, (_, _, text)) in kept.iter().enumerate() {
        let mut line = engine.line(text);
        let number = first_number + k as i64;
        if opts.numbers != Numbers::None && number % step == 0 {
            let blank = line.pieces.is_empty();
            if !blank || opts.numberblanklines {
                line.number = Some(number);
            }
        }
        out.push(line);
    }
    let _ = (engine.base, engine.inline);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Tt;
    impl Measure for Tt {
        fn quad_ex(&mut self, _font: FontState) -> (f64, f64) {
            (10.5, 4.3)
        }
        fn char_widths(&mut self, _font: FontState, text: &str) -> Vec<f64> {
            text.chars().map(|_| 5.25).collect()
        }
        fn space(&mut self, _font: FontState) -> f64 {
            5.25
        }
    }

    fn xs(line: &OutLine) -> Vec<(String, f64)> {
        let mut x = 0.0;
        let mut out = Vec::new();
        for p in &line.pieces {
            match p {
                Piece::Kern(k) => x += k,
                Piece::Token { text, width, offsets, .. } => {
                    for (c, o) in text.chars().zip(offsets) {
                        out.push((c.to_string(), x + o));
                    }
                    x += width;
                }
                Piece::Break => {}
            }
        }
        out
    }

    #[test]
    fn fixed_columns_centre_each_token_in_its_columns() {
        let opts = options_at("", 0, Some("basicstyle=\\ttfamily"));
        let lines = layout(&["int x;", "    y"], &opts, FontState::default(), 10, false, &mut Tt);
        let first = xs(&lines[0]);
        // "int": a 3-column box of 6.3pt columns, 4 hss glues of 0.7875pt.
        assert!((first[0].1 - 0.7875).abs() < 1e-9, "{first:?}");
        assert!((first[1].1 - (0.7875 * 2.0 + 5.25)).abs() < 1e-9);
        // "x" at column 4 (after the one-column space box), alone in its box.
        let x = first.iter().find(|(c, _)| c == "x").unwrap().1;
        assert!((x - (4.0 * 6.3 + (6.3 - 5.25) / 2.0)).abs() < 1e-9, "{x}");
        let second = xs(&lines[1]);
        assert!((second[0].1 - (4.0 * 6.3 + (6.3 - 5.25) / 2.0)).abs() < 1e-9, "{second:?}");
    }

    #[test]
    fn nolig_characters_add_an_element_to_the_token_being_built() {
        let opts = options_at("", 0, Some("basicstyle=\\ttfamily"));
        let lines = layout(&["ab,<"], &opts, FontState::default(), 10, false, &mut Tt);
        let line = xs(&lines[0]);
        // "ab" + `\lst@nolig` (from ","): 2 columns, 4 glues of 2.1/4.
        assert!((line[0].1 - 2.1 / 4.0).abs() < 1e-9, "{line:?}");
        // ",<" is one other token: ",", `\lst@nolig`, "<" in 2 columns, 4 glues.
        assert!((line[2].1 - (12.6 + 0.525)).abs() < 1e-9, "{line:?}");
        assert!((line[3].1 - (12.6 + 3.0 * 0.525 + 5.25)).abs() < 1e-9, "{line:?}");
        let parts = break_line(&[Piece::Kern(1.0), Piece::Break, Piece::Kern(2.0)], 10.0, 5.0);
        assert_eq!(parts.len(), 1);
    }

    #[test]
    fn flexible_columns_keep_leading_spaces_as_lost_space() {
        let opts = options_at("", 0, Some("basicstyle=\\ttfamily,columns=flexible"));
        let lines = layout(&["    int x"], &opts, FontState::default(), 10, false, &mut Tt);
        let line = xs(&lines[0]);
        assert!((line[0].1 - 4.0 * 0.45 * 10.5).abs() < 1e-9, "{line:?}");
    }

    #[test]
    fn keywords_comments_and_strings_are_classified() {
        let opts = options_at("\\lstset{language=C}", 100, Some("basicstyle=\\ttfamily"));
        let lines = layout(&["int s = \"if\"; // for"], &opts, FontState::default(), 10, false, &mut Tt);
        let classes: Vec<(String, Class)> = lines[0]
            .pieces
            .iter()
            .filter_map(|p| match p {
                Piece::Token { text, class, .. } => Some((text.clone(), *class)),
                _ => None,
            })
            .collect();
        assert_eq!(classes[0], ("int".to_string(), Class::Keyword(1)));
        assert!(classes.iter().any(|(t, c)| t == "if" && *c == Class::String));
        assert!(classes.iter().any(|(t, c)| t == "for" && *c == Class::Comment));
        assert!(classes.iter().any(|(t, c)| t == "s" && *c == Class::Identifier));
    }

    #[test]
    fn options_follow_lstset_styles_and_own_keys() {
        let src = "\\lstdefinestyle{mine}{numbers=left,basicstyle=\\ttfamily\\small}\n\\lstset{style=mine,tabsize=4}\n% \\lstset{tabsize=2}\nX";
        let opts = options_at(src, src.len(), Some("frame=single,language={[3]Python}"));
        assert_eq!(opts.numbers, Numbers::Left);
        assert_eq!(opts.tabsize, 4);
        assert!(opts.frame.left && opts.frame.top);
        assert_eq!(opts.language.map(|l| l.keywords.contains(&"nonlocal")), Some(true));
        let font = opts.basicstyle.apply(FontState::default(), 10);
        assert!(font.mono);
        assert_eq!(font.size_cpt, 900);
        assert_eq!(SkipSpec::parse("6pt plus 2pt minus 1pt"), Some(SkipSpec { natural: Dimen::pt(6.0), stretch: 2.0, shrink: 1.0 }));
    }

    #[test]
    fn trailing_empty_lines_and_gobble() {
        let opts = options_at("", 0, Some("gobble=2"));
        let kept = kept_lines(&["  ab", "", "  c", "", "   "], &opts);
        assert_eq!(kept.iter().map(|k| k.2).collect::<Vec<_>>(), vec!["ab", "", "c"]);
    }
}
