//! Manual bibliographies and citations: `thebibliography`, `\bibitem`,
//! `\cite`, `\nocite`, and natbib's citation commands.
//!
//! A `\bibitem`'s citation label depends only on how many earlier `\bibitem`s
//! precede it (or its own optional-label override) — never on page numbers or
//! line breaks. That makes it fundamentally simpler than `\label`/`\ref`,
//! which need the page-aware two-pass resolution in `layout.rs`: a
//! [`prescan`] of the token stream the parser walks (after macro expansion) is enough to
//! resolve every `\cite` in one pass, even one that appears (as citations
//! normally do) before the `thebibliography` it points into. This module is
//! deliberately self-contained and does not touch that separate label/ref
//! resolution pass.
//!
//! The text of a citation follows the macros that produce it:
//!
//! * the kernel (`latex.ltx` 17754-17800): `\@citex` joins the labels with
//!   `,\penalty\@m\ ` and `\@cite` wraps them as `[<labels>, <note>]`;
//! * natbib 8.31b (`natbib.sty`): `\NAT@citexnum` for numerical citations
//!   (brackets, `sort`/`compress`, textual `\citet` as `Name [n]`) and
//!   `\NAT@citex` for author-year citations, with the punctuation of the
//!   package options, `\bibpunct`, `\setcitestyle` and the BibTeX style's
//!   `\bibstyle@<name>`; `\bibitem[Name(Year)Long names]{key}` supplies the
//!   names and year (`\NAT@bare`).
//!
//! In the returned runs a space character is TeX's interword glue (`\ `),
//! a `~` a tie; everything else is literal text.
//!
//! BibTeX `.bib` files are out of scope: a project's `<jobname>.bbl` (the
//! file BibTeX writes) is read in place of `\bibliography` by `parser.rs`.

use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};

use crate::diagnostics::Diagnostic;
use crate::lexer::{Token, TokenKind};
use crate::parser::{Inline, TextStyle};
use crate::Span;

/// natbib's `\bibitem[Name(Year)Long names]{key}` data (`\NAT@bare`):
/// the short and long author lists and the date (year plus extra label).
#[derive(Debug, Clone, PartialEq)]
struct NatNames {
    short: String,
    long: String,
    date: String,
}

/// One `\bibitem`.
#[derive(Debug, Clone)]
struct BibItem {
    /// The kernel's label: the `enumiv` number of a plain `\bibitem{key}`
    /// or the verbatim optional argument.
    kernel_label: String,
    /// natbib's `\NAT@num`: `\c@NAT@ctr` (every entry counts), or the whole
    /// optional argument when it has neither `(year)` nor `, year`.
    natbib_num: String,
    /// `None`: a label natbib cannot read author-year data from.
    names: Option<NatNames>,
}

/// Every `\bibitem` found by [`prescan`], in document order, plus a
/// key → item lookup for `\cite`.
#[derive(Debug, Default)]
pub struct Bibliography {
    items: Vec<BibItem>,
    keys: HashMap<String, usize>,
}

impl Bibliography {
    fn push(&mut self, key: String, item: BibItem, span: Span, diags: &mut Vec<Diagnostic>) {
        let index = self.items.len();
        self.items.push(item);
        if key.is_empty() {
            return;
        }
        if self.keys.insert(key.clone(), index).is_some() {
            diags.push(Diagnostic::warning(
                format!("duplicate \\bibitem{{{key}}}; the second definition wins"),
                Some(span),
                Some("used the later entry's label for \\cite".into()),
            ));
        }
    }

    /// The kernel label of the `index`th (0-based) `\bibitem` in document
    /// order, consulted by the real parse — see `parser::P::bib_cursor` —
    /// which walks the same literal `\bibitem`s this pre-scan already
    /// numbered.
    pub fn label_at(&self, index: usize) -> Option<&str> {
        self.items.get(index).map(|item| item.kernel_label.as_str())
    }

    /// The label the `index`th `\bibitem` sets in the list under `style`:
    /// the kernel's `\@biblabel{..}` (`[<label>]`), natbib's numerical
    /// `\bibnumfmt{\NAT@num}` (`[<n>]`), or nothing for natbib author-year
    /// lists (`\NAT@biblabel` is `\hfill`).
    pub fn list_label(&self, index: usize, style: &CiteStyle) -> Option<String> {
        let item = self.items.get(index)?;
        if !style.natbib {
            return Some(label_bracket(&item.kernel_label));
        }
        if self.numbers(style) {
            Some(label_bracket(&item.natbib_num))
        } else {
            Some(String::new())
        }
    }

    pub(crate) fn resolve(&self, key: &str) -> Option<&str> {
        self.item(key).map(|item| item.kernel_label.as_str())
    }

    fn item(&self, key: &str) -> Option<&BibItem> {
        self.keys.get(key).and_then(|&index| self.items.get(index))
    }

    /// natbib's numerical mode as a document settles after enough runs: an
    /// author-year bibliography with an entry lacking `(year)` data makes
    /// `\NAT@force@numbers` switch to numbers (natbib.sty 981-990).
    pub fn numbers(&self, style: &CiteStyle) -> bool {
        style.numbers || (style.natbib && self.items.iter().any(|item| item.names.is_none()))
    }

    /// Whether natbib will force numerical citations on an author-year
    /// document ("Bibliography not compatible with author-year citations").
    pub fn forces_numbers(&self, style: &CiteStyle) -> bool {
        style.natbib && !style.numbers && self.numbers(style)
    }
}

/// Scans a token stream for every `\bibitem` inside a `thebibliography`
/// environment, in order. A plain `\bibitem{key}` numbers sequentially;
/// `\bibitem[label]{key}` uses `label` verbatim for the kernel and does not
/// consume an `enumiv` number (`\@lbibitem`), while natbib counts every
/// entry. The parser passes the same expanded stream it walks, so
/// `P::bib_cursor` meets exactly these `\bibitem`s (including one a macro
/// produced, and never one under `\iffalse`).
pub fn prescan<T: Borrow<Token>>(tokens: &[T], diags: &mut Vec<Diagnostic>) -> Bibliography {
    let mut bibliography = Bibliography::default();
    let mut in_bibliography = false;
    let mut next_number: u32 = 1;
    let mut natbib_counter: u32 = 0;
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i].borrow().kind {
            TokenKind::Command(name) if name == "begin" || name == "end" => {
                let is_begin = name == "begin";
                match group_text(tokens, i + 1) {
                    Some((environment, after)) => {
                        if environment.trim() == "thebibliography" {
                            in_bibliography = is_begin;
                        }
                        i = after;
                    }
                    None => i += 1,
                }
            }
            TokenKind::Command(name) if name == "bibitem" && in_bibliography => {
                let span = tokens[i].borrow().span;
                let mut cursor = i + 1;
                let mut label_override = None;
                if let Some((text, after)) = optional_bracket_text(tokens, cursor) {
                    label_override = Some(text);
                    cursor = after;
                }
                if let Some((key, after)) = group_text(tokens, cursor) {
                    cursor = after;
                    natbib_counter += 1;
                    let kernel_label = label_override.clone().unwrap_or_else(|| {
                        let n = next_number;
                        next_number += 1;
                        n.to_string()
                    });
                    let (natbib_num, names) = match label_override.as_deref() {
                        Some(label) if !label.is_empty() => natbib_label(label, natbib_counter),
                        _ => (natbib_counter.to_string(), None),
                    };
                    let item = BibItem {
                        kernel_label,
                        natbib_num,
                        names,
                    };
                    bibliography.push(key.trim().to_string(), item, span, diags);
                }
                i = cursor;
            }
            _ => i += 1,
        }
    }
    bibliography
}

/// The BibTeX style named by the first `\bibliographystyle{..}`: LaTeX
/// writes it to the `.aux` file, whose `\bibstyle` line natbib reads at
/// `\begin{document}` — before any citation, wherever the command sits.
pub fn bibliography_style<T: Borrow<Token>>(tokens: &[T]) -> Option<String> {
    let at = tokens
        .iter()
        .position(|t| matches!(&t.borrow().kind, TokenKind::Command(name) if name == "bibliographystyle"))?;
    group_text(tokens, at + 1).map(|(name, _)| name.trim().to_string())
}

/// natbib's reading of a `\bibitem` optional argument (`\NAT@bare`,
/// `\NAT@apalk`): `Name(Year)Long names` gives names and a date; without a
/// parenthesis `Name, Year` does too, and anything else is the citation
/// number itself. Returns `(\NAT@num, names)`.
fn natbib_label(label: &str, counter: u32) -> (String, Option<NatNames>) {
    if let Some((short, rest)) = label.split_once('(') {
        let (date, long) = rest.split_once(')').unwrap_or((rest, ""));
        let short = short.to_string();
        let long = if long.is_empty() { short.clone() } else { long.to_string() };
        return (
            counter.to_string(),
            Some(NatNames {
                short,
                long,
                date: date.to_string(),
            }),
        );
    }
    match label.split_once(", ") {
        Some((name, year)) if !year.trim().is_empty() => (
            counter.to_string(),
            Some(NatNames {
                short: name.to_string(),
                long: name.to_string(),
                date: year.split(", ").next().unwrap_or(year).to_string(),
            }),
        ),
        _ => (label.to_string(), None),
    }
}

/// The text inside the next `{...}` group starting at (after skipping
/// spaces/comments from) `i`, and the index just past its closing brace.
/// `None` if `i` is not followed by a brace group — a malformed `\bibitem`
/// or `\begin`/`\end` is left for the real parse's own diagnostics.
fn group_text<T: Borrow<Token>>(tokens: &[T], mut i: usize) -> Option<(String, usize)> {
    while matches!(
        tokens.get(i).map(|t| &t.borrow().kind),
        Some(TokenKind::Space | TokenKind::Comment)
    ) {
        i += 1;
    }
    if !matches!(tokens.get(i).map(|t| &t.borrow().kind), Some(TokenKind::LBrace)) {
        return None;
    }
    i += 1;
    let mut depth = 1usize;
    let mut text = String::new();
    while i < tokens.len() {
        match &tokens[i].borrow().kind {
            TokenKind::LBrace => depth += 1,
            TokenKind::RBrace => {
                depth -= 1;
                if depth == 0 {
                    return Some((text, i + 1));
                }
            }
            TokenKind::Word(word) | TokenKind::Command(word) => text.push_str(word),
            TokenKind::Space | TokenKind::ParBreak => text.push(' '),
            _ => {}
        }
        i += 1;
    }
    None
}

/// The text inside a `[...]` immediately at (after skipping spaces/comments
/// from) `i`, mirroring `parser::P::optional_bracket_argument`'s word-based
/// bracket matching (brackets are ordinary lexer word characters, never
/// their own token kind) against a plain token slice instead of the live
/// parse cursor. Braces inside the brackets are dropped and
/// `\natexlab{x}` (BibTeX's extra year label) reads as `x`.
fn optional_bracket_text<T: Borrow<Token>>(tokens: &[T], mut i: usize) -> Option<(String, usize)> {
    while matches!(
        tokens.get(i).map(|t| &t.borrow().kind),
        Some(TokenKind::Space | TokenKind::Comment)
    ) {
        i += 1;
    }
    let TokenKind::Word(first) = &tokens.get(i)?.borrow().kind else {
        return None;
    };
    if !first.starts_with('[') {
        return None;
    }
    let mut raw = first.clone();
    let mut found = raw.contains(']');
    i += 1;
    let mut depth = 0usize;
    while !found && i < tokens.len() {
        match &tokens[i].borrow().kind {
            TokenKind::Word(word) => {
                if depth == 0 {
                    if let Some(close) = word.find(']') {
                        raw.push_str(&word[..=close]);
                        found = true;
                        i += 1;
                        break;
                    }
                }
                raw.push_str(word);
            }
            TokenKind::Space | TokenKind::ParBreak => raw.push(' '),
            TokenKind::LBrace => depth += 1,
            TokenKind::RBrace => depth = depth.saturating_sub(1),
            TokenKind::Command(word) if word == "natexlab" => {}
            TokenKind::Command(word) => {
                raw.push('\\');
                raw.push_str(word);
            }
            _ => {}
        }
        i += 1;
    }
    if !found {
        return None;
    }
    let content = raw
        .strip_prefix('[')
        .unwrap_or(&raw)
        .split_once(']')
        .map_or(raw.as_str(), |(inside, _)| inside)
        .to_string();
    Some((content, i))
}

/// Citation punctuation and mode: the kernel's, or natbib's after its
/// package options, `\bibpunct`, `\setcitestyle`, `\citestyle` and the
/// BibTeX style.
#[derive(Debug, Clone, PartialEq)]
pub struct CiteStyle {
    /// natbib is loaded.
    pub natbib: bool,
    /// `\ifNAT@numbers`.
    pub numbers: bool,
    /// `\ifNAT@super`: superscript numbers.
    pub superscript: bool,
    pub open: String,
    pub close: String,
    /// `\NAT@sep` between citations.
    pub sep: String,
    /// `\NAT@aysep` between author and year (`\citep`).
    pub aysep: String,
    /// `\NAT@yrsep` between years of one author.
    pub yrsep: String,
    /// `\NAT@cmt` before a post-note.
    pub cmt: String,
    pub sort: bool,
    pub compress: bool,
    /// `\bibstyle` is still live (no option set `nobibstyle`): the BibTeX
    /// style's `\bibstyle@<name>` punctuation applies at `\begin{document}`.
    pub bibstyle: bool,
    /// `sectionbib`: report/book bibliographies are `\section*{\bibname}`.
    pub sectionbib: bool,
    /// `openbib`: `\newblock` is `\par`.
    pub openbib: bool,
    /// `longnamesfirst`: a key's first citation gives every author.
    pub longnamesfirst: bool,
}

impl Default for CiteStyle {
    fn default() -> Self {
        CiteStyle {
            natbib: false,
            numbers: true,
            superscript: false,
            open: "[".into(),
            close: "]".into(),
            sep: ",".into(),
            aysep: ",".into(),
            yrsep: ",".into(),
            cmt: ", ".into(),
            sort: false,
            compress: false,
            bibstyle: false,
            sectionbib: false,
            openbib: false,
            longnamesfirst: false,
        }
    }
}

/// natbib's options in declaration order (`\ProcessOptions` runs them in
/// this order, whatever order the document lists them in).
const NATBIB_OPTIONS: &[&str] = &[
    "numbers",
    "super",
    "authoryear",
    "round",
    "square",
    "angle",
    "curly",
    "comma",
    "semicolon",
    "colon",
    "nobibstyle",
    "bibstyle",
    "openbib",
    "sectionbib",
    "sort",
    "compress",
    "sort&compress",
    "longnamesfirst",
];

impl CiteStyle {
    /// `\usepackage[<options>]{natbib}`: author-year, round brackets and
    /// semicolons unless the options say otherwise. Returns the style and
    /// the options it did not recognise.
    pub fn natbib(options: &str) -> (CiteStyle, Vec<String>) {
        let mut style = CiteStyle {
            natbib: true,
            numbers: false,
            open: "(".into(),
            close: ")".into(),
            sep: ";".into(),
            bibstyle: true,
            ..CiteStyle::default()
        };
        let given: Vec<&str> = options.split(',').map(str::trim).filter(|o| !o.is_empty()).collect();
        for option in NATBIB_OPTIONS {
            if given.contains(option) {
                style.option(option);
            }
        }
        let unknown = given
            .iter()
            .filter(|o| !NATBIB_OPTIONS.contains(o) && !matches!(**o, "mcite" | "merge" | "elide" | "nonamebreak"))
            .map(|o| o.to_string())
            .collect();
        (style, unknown)
    }

    fn option(&mut self, option: &str) {
        match option {
            "numbers" => {
                self.numbers = true;
                self.option("square");
                self.option("comma");
            }
            "super" => {
                self.superscript = true;
                self.numbers = true;
                self.open.clear();
                self.close.clear();
                self.bibstyle = false;
            }
            "authoryear" => {
                self.numbers = false;
                self.option("round");
                self.option("semicolon");
                self.bibstyle = true;
            }
            "round" => self.brackets("(", ")"),
            "square" => self.brackets("[", "]"),
            "angle" => self.brackets("<", ">"),
            "curly" => self.brackets("{", "}"),
            "comma" => {
                self.sep = ",".into();
                self.bibstyle = false;
            }
            "semicolon" | "colon" => {
                self.sep = ";".into();
                self.bibstyle = false;
            }
            "nobibstyle" => self.bibstyle = false,
            "bibstyle" => self.bibstyle = true,
            "openbib" => self.openbib = true,
            "sectionbib" => self.sectionbib = true,
            "sort" => self.sort = true,
            "compress" => self.compress = true,
            "sort&compress" => {
                self.sort = true;
                self.compress = true;
            }
            "longnamesfirst" => self.longnamesfirst = true,
            _ => {}
        }
    }

    fn brackets(&mut self, open: &str, close: &str) {
        self.open = open.into();
        self.close = close.into();
        self.bibstyle = false;
    }

    /// `\bibpunct[<cmt>]{open}{close}{sep}{mode}{aysep}{yrsep}`.
    pub fn bibpunct(&mut self, cmt: Option<&str>, args: &[String]) {
        let [open, close, sep, mode, aysep, yrsep] = args else {
            return;
        };
        self.open = open.clone();
        self.close = close.clone();
        self.sep = sep.clone();
        self.numbers = false;
        match mode.trim() {
            "n" => {
                self.numbers = true;
                self.superscript = false;
            }
            "s" => {
                self.numbers = true;
                self.superscript = true;
            }
            _ => {}
        }
        self.aysep = aysep.clone();
        self.yrsep = yrsep.clone();
        self.cmt = cmt.unwrap_or(", ").to_string();
        self.bibstyle = false;
    }

    /// `\setcitestyle{<keywords and key=value>}` (the argument with its
    /// braces: `\@for` splits at top-level commas and a braced value keeps
    /// its spaces, `notesep={: }`).
    pub fn setcitestyle(&mut self, list: &str) {
        for entry in split_top_level(list).iter().map(|e| e.trim()).filter(|e| !e.is_empty()) {
            match entry.split_once('=') {
                Some((key, value)) => {
                    let value = value.trim();
                    let value = value
                        .strip_prefix('{')
                        .and_then(|v| v.strip_suffix('}'))
                        .unwrap_or(value)
                        .to_string();
                    match key.trim() {
                        "open" => self.open = value,
                        "close" => self.close = value,
                        "aysep" => self.aysep = value,
                        "yysep" => self.yrsep = value,
                        "notesep" => self.cmt = value,
                        "citesep" => self.sep = value,
                        _ => {}
                    }
                }
                None => match entry {
                    "round" | "square" | "angle" | "curly" | "semicolon" | "colon" | "comma" => self.option(entry),
                    "authoryear" => self.numbers = false,
                    "numbers" => {
                        self.numbers = true;
                        self.superscript = false;
                    }
                    "super" => {
                        self.numbers = true;
                        self.superscript = true;
                    }
                    _ => {}
                },
            }
        }
        self.bibstyle = false;
    }

    /// `\citestyle{<name>}` (always) or, while `\bibstyle` is live, the
    /// `\bibliographystyle{<name>}` BibTeX writes to the `.aux` file.
    pub fn named_style(&mut self, name: &str, explicit: bool) {
        if !self.natbib || !(explicit || self.bibstyle) {
            return;
        }
        let punct: [&str; 6] = match name.trim() {
            "chicago" | "copernicus" | "egu" | "egs" | "pass" | "anngeo" | "nlinproc" => ["(", ")", ";", "a", ",", ","],
            "named" => ["[", "]", ";", "a", ",", ","],
            "agu" => ["[", "]", ";", "a", ",", ",~"],
            "agsm" | "kluwer" => ["(", ")", ",", "a", "", ","],
            "dcu" => ["(", ")", ";", "a", ";", ","],
            "aa" => ["(", ")", ";", "a", "", ","],
            "cospar" => ["/", "/", ",", "n", "", ""],
            "esa" => ["(Ref.~", ")", ",", "n", "", ""],
            "nature" => ["", "", ",", "s", "", ","],
            "plain" | "alpha" | "abbrv" | "unsrt" => ["[", "]", ",", "n", "", ","],
            "plainnat" | "abbrvnat" | "unsrtnat" => ["[", "]", ",", "a", ",", ","],
            _ => return,
        };
        let args: Vec<String> = punct.iter().map(|s| s.to_string()).collect();
        self.bibpunct(None, &args);
        if !explicit {
            self.bibstyle = true;
        }
    }
}

/// `list` split at commas outside braces.
fn split_top_level(list: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
    for (at, c) in list.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                parts.push(&list[start..at]);
                start = at + 1;
            }
            _ => {}
        }
    }
    parts.push(&list[start..]);
    parts
}

/// The text of an argument's tokens with its inner braces kept (unlike
/// the parser's `token_text`), for key=value lists whose braces matter.
pub fn braced_text<'t>(tokens: impl Iterator<Item = &'t Token>) -> String {
    let mut text = String::new();
    for token in tokens {
        match &token.kind {
            TokenKind::Word(word) => text.push_str(word),
            TokenKind::Command(name) => {
                text.push('\\');
                text.push_str(name);
            }
            TokenKind::LBrace => text.push('{'),
            TokenKind::RBrace => text.push('}'),
            TokenKind::Space | TokenKind::ParBreak => text.push(' '),
            _ => {}
        }
    }
    text
}

/// Which citation command, with its natbib `\NAT@swa`/`\NAT@par`/
/// `\NAT@ctype` settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CiteKind {
    Cite,
    Citet,
    Citep,
    Citealt,
    Citealp,
    Citeauthor,
    Citeyear,
    Citeyearpar,
    Citenum,
}

impl CiteKind {
    pub fn from_command(name: &str) -> Option<(CiteKind, bool)> {
        let capital = name.starts_with('C');
        let kind = match name.to_ascii_lowercase().as_str() {
            "cite" if !capital => CiteKind::Cite,
            "citet" => CiteKind::Citet,
            "citep" => CiteKind::Citep,
            "citealt" => CiteKind::Citealt,
            "citealp" => CiteKind::Citealp,
            "citeauthor" => CiteKind::Citeauthor,
            "citeyear" if !capital => CiteKind::Citeyear,
            "citeyearpar" if !capital => CiteKind::Citeyearpar,
            "citenum" if !capital => CiteKind::Citenum,
            _ => return None,
        };
        Some((kind, capital))
    }
}

/// One citation command as read from the source.
#[derive(Debug, Clone)]
pub struct CiteRequest {
    pub kind: CiteKind,
    /// `\Citet`...: the first author's initial is upper-cased.
    pub capital: bool,
    /// `\citet*`: full author lists.
    pub star: bool,
    /// The optional arguments in order (natbib: one is the post-note, two
    /// are pre- and post-note; the kernel's `\cite` has one note).
    pub optionals: Vec<String>,
    pub keys: Vec<String>,
}

/// What one citation resolved to: the structured record the compiler
/// publishes next to the paragraph runs (hyperref links it to `cite.<key>`).
#[derive(Debug, Clone, PartialEq)]
pub struct Citation {
    /// The command and its arguments.
    pub span: Span,
    /// The command name (`cite`, `citet`, `Citep`, ...).
    pub command: String,
    pub keys: Vec<String>,
    /// hyperref's destination for each key (`cite.<key>`), `None` when
    /// the key is undefined.
    pub targets: Vec<Option<String>>,
    /// The typeset text (a space is interword glue).
    pub text: String,
}

/// Citation state carried through one parse: the style and the keys
/// already cited (for `longnamesfirst`).
#[derive(Debug, Default)]
pub struct Citer {
    pub style: CiteStyle,
    cited: HashSet<String>,
}

/// A piece of citation output: text, bold for LaTeX's undefined-citation
/// markers.
#[derive(Debug, Default)]
struct Out {
    runs: Vec<(String, bool)>,
}

impl Out {
    fn text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        match self.runs.last_mut() {
            Some((last, false)) => last.push_str(text),
            _ => self.runs.push((text.to_string(), false)),
        }
    }

    fn bold(&mut self, text: &str) {
        self.runs.push((text.to_string(), true));
    }

    fn plain(&self) -> String {
        self.runs.iter().map(|(t, _)| t.as_str()).collect()
    }
}

/// The year and extra label of a natbib date (`\NAT@parse@date`): up to
/// four leading non-letters, then the first letter (`?` without one).
fn split_date(date: &str) -> (String, String) {
    let mut year = String::new();
    for (count, c) in date.chars().enumerate() {
        if c.is_alphabetic() {
            return (year, c.to_string());
        }
        if count == 4 {
            break;
        }
        year.push(c);
    }
    (year, "?".to_string())
}

fn upper_first(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn is_number(text: &str) -> Option<i64> {
    (!text.is_empty() && text.chars().all(|c| c.is_ascii_digit()))
        .then(|| text.parse::<i64>().ok())
        .flatten()
        .filter(|n| *n > 0)
}

impl Citer {
    pub fn new(style: CiteStyle) -> Self {
        Citer { style, cited: HashSet::new() }
    }

    /// The runs and record of one citation command. Undefined keys warn
    /// with LaTeX's (or natbib's) message.
    pub fn cite(
        &mut self,
        request: &CiteRequest,
        command: &str,
        bibliography: &Bibliography,
        span: Span,
        diags: &mut Vec<Diagnostic>,
    ) -> (Vec<Inline>, Citation) {
        let mut keys = request.keys.clone();
        let natbib = self.style.natbib;
        let numbers = !natbib || bibliography.numbers(&self.style) || request.kind == CiteKind::Citenum;
        if natbib && numbers && self.style.sort {
            keys = sort_keys(&keys, bibliography);
        }
        for key in &keys {
            if bibliography.item(key).is_none() {
                diags.push(Diagnostic::warning(
                    format!("Citation `{key}' undefined"),
                    Some(span),
                    Some("rendered '?' in place of the undefined citation".into()),
                ));
            }
        }
        let out = if !natbib {
            kernel_cite(&keys, request.optionals.first().map(String::as_str), bibliography)
        } else if numbers {
            self.natbib_numbers(request, &keys, bibliography)
        } else {
            self.natbib_authoryear(request, &keys, bibliography)
        };
        for key in &keys {
            self.cited.insert(key.clone());
        }
        let citation = Citation {
            span,
            command: command.to_string(),
            targets: keys
                .iter()
                .map(|key| bibliography.item(key).map(|_| format!("cite.{key}")))
                .collect(),
            keys,
            text: out.plain(),
        };
        let mut inlines = Vec::new();
        if !natbib {
            // The kernel's runs keep their historical split (`[`, labels,
            // `, `, note, `]`) for the compiler's own layout.
            for (index, (text, bold)) in out.runs.iter().enumerate() {
                let style = if *bold { TextStyle::BOLD } else { TextStyle::default() };
                for piece in split_kernel_runs(text) {
                    inlines.push(text_run(&piece, span, style, index == 0 && inlines.is_empty()));
                }
            }
        } else {
            for (index, (text, bold)) in out.runs.iter().enumerate() {
                let style = if *bold { TextStyle::BOLD } else { TextStyle::default() };
                inlines.push(text_run(text, span, style, index == 0));
            }
        }
        (inlines, citation)
    }

    /// Author names for one citation (`\NAT@nm`): the long list under a
    /// starred command or on a key's first citation with `longnamesfirst`.
    fn names(&self, request: &CiteRequest, key: &str, names: &NatNames) -> String {
        let full = request.star || (self.style.longnamesfirst && !self.cited.contains(key));
        let name = if full { &names.long } else { &names.short };
        if request.capital {
            upper_first(name)
        } else {
            name.clone()
        }
    }

    /// `\NAT@citexnum` wrapped by `\NAT@citenum` (or `\NAT@citesuper`).
    fn natbib_numbers(&self, request: &CiteRequest, keys: &[String], bibliography: &Bibliography) -> Out {
        let s = &self.style;
        let (swa, par, ctype) = natbib_flags(request.kind, true);
        let (pre, post) = notes(&request.optionals);
        let open = if par { s.open.as_str() } else { "" };
        let close = if par { s.close.as_str() } else { "" };
        let space = if s.superscript { "" } else { " " };
        let mut body = Out::default();
        let mut citea = String::new();
        if swa {
            let mut num = "-1".to_string();
            let mut last_yr: Option<String> = None;
            for key in keys {
                let Some(item) = bibliography.item(key) else {
                    // natbib.sty 385: `{\reset@font\bfseries?}` ends its
                    // line, so a space follows the mark.
                    body.bold("?");
                    body.text(" ");
                    continue;
                };
                let last_num = std::mem::replace(&mut num, item.natbib_num.clone());
                if ctype == 2 {
                    body.text(&citea);
                    match item.names.as_ref().map(|n| n.date.as_str()).filter(|d| !d.is_empty()) {
                        Some(date) => body.text(date),
                        None => body.bold("(year?)"),
                    }
                } else if s.compress {
                    let nm = is_number(&num).unwrap_or(-2);
                    let last = is_number(&last_num).unwrap_or(-1);
                    if nm == last {
                        if let Some(pending) = last_yr.take() {
                            body.text(&pending);
                        }
                        body.text(&citea);
                        body.text(&num);
                    } else if nm == last + 1 {
                        last_yr = Some(match last_yr {
                            None => format!("{citea}{num}"),
                            Some(_) => format!("\u{2013}{num}"),
                        });
                    } else {
                        if let Some(pending) = last_yr.take() {
                            body.text(&pending);
                        }
                        body.text(&citea);
                        body.text(&num);
                    }
                } else {
                    body.text(&citea);
                    body.text(&num);
                }
                citea = format!("{}{space}", s.sep);
            }
            if let Some(pending) = last_yr {
                body.text(&pending);
            }
            let mut out = Out::default();
            if s.superscript && request.kind != CiteKind::Citenum {
                // `\NAT@citesuper`: `\textsuperscript{open list close}`.
                if !pre.is_empty() {
                    out.text(&format!("{pre} "));
                }
                out.text(open);
                out.runs.extend(body.runs);
                out.text(close);
                if !post.is_empty() {
                    out.text(&format!(" {post}"));
                }
            } else {
                out.text(open);
                if !pre.is_empty() {
                    out.text(&format!("{pre} "));
                }
                out.runs.extend(body.runs);
                if !post.is_empty() {
                    out.text(&format!("{}{post}", s.cmt));
                }
                out.text(close);
            }
            return merge(out);
        }
        let mut last_nm: Option<Option<String>> = None;
        for key in keys {
            let Some(item) = bibliography.item(key) else {
                body.bold("?");
                body.text(" ");
                continue;
            };
            let nm = item.names.as_ref().map(|n| self.names(request, key, n));
            match ctype {
                0 => {
                    if last_nm.as_ref() == Some(&nm) {
                        body.text(&format!("{}{space}", s.yrsep));
                    } else {
                        body.text(&citea);
                        match nm.as_deref().filter(|n| !n.is_empty()) {
                            Some(name) => body.text(name),
                            None => body.bold("(author?)"),
                        }
                        body.text(&format!(" {open}"));
                    }
                    if !pre.is_empty() {
                        body.text(&format!("{pre} "));
                    }
                    body.text(&item.natbib_num);
                    citea = format!("{close}{} ", s.sep);
                }
                1 => {
                    body.text(&citea);
                    match nm.as_deref().filter(|n| !n.is_empty()) {
                        Some(name) => body.text(name),
                        None => body.bold("(author?)"),
                    }
                    citea = format!("{} ", s.sep);
                }
                _ => {
                    body.text(&citea);
                    match item.names.as_ref().map(|n| n.date.as_str()).filter(|d| !d.is_empty()) {
                        Some(date) => body.text(date),
                        None => body.bold("(year?)"),
                    }
                    citea = format!("{} ", s.sep);
                }
            }
            last_nm = Some(nm);
        }
        if ctype == 0 && !post.is_empty() {
            body.text(&format!("{}{post}", s.cmt));
        }
        body.text(close);
        merge(body)
    }

    /// `\NAT@citex` wrapped by `\NAT@cite`.
    fn natbib_authoryear(&self, request: &CiteRequest, keys: &[String], bibliography: &Bibliography) -> Out {
        let s = &self.style;
        let (swa, par, ctype) = natbib_flags(request.kind, !request.optionals.is_empty());
        let (pre, post) = notes(&request.optionals);
        let open = if par { s.open.as_str() } else { "" };
        let close = if par { s.close.as_str() } else { "" };
        let mut body = Out::default();
        let mut citea = String::new();
        let mut nm: Option<String> = None;
        let mut year: Option<String> = None;
        let mut date = String::new();
        for key in keys {
            let Some(names) = bibliography.item(key).and_then(|item| item.names.as_ref()) else {
                body.text(&citea);
                body.bold("?");
                date.clear();
                citea = format!("{} ", s.sep);
                continue;
            };
            let last_nm = nm.replace(self.names(request, key, names));
            date = names.date.clone();
            let (this_year, exlab) = split_date(&date);
            let last_yr = year.replace(this_year.clone());
            let name = nm.clone().unwrap_or_default();
            match ctype {
                0 if date.is_empty() => {
                    body.text(&citea);
                    body.text(&name);
                }
                0 if last_nm.as_ref() == Some(&name) => {
                    body.text(&s.yrsep);
                    if last_yr.as_ref() == Some(&this_year) {
                        body.text(&exlab);
                    } else {
                        body.text(&format!(" {date}"));
                    }
                }
                0 if swa => {
                    body.text(&citea);
                    body.text(&format!("{name}{} {date}", s.aysep));
                }
                0 => {
                    body.text(&citea);
                    body.text(&format!("{name} {open}"));
                    if !pre.is_empty() {
                        body.text(&format!("{pre} "));
                    }
                    body.text(&date);
                }
                1 => {
                    body.text(&citea);
                    body.text(&name);
                }
                _ => {
                    body.text(&citea);
                    body.text(&date);
                }
            }
            citea = if swa || date.is_empty() {
                format!("{} ", s.sep)
            } else {
                format!("{close}{} ", s.sep)
            };
        }
        let mut out = Out::default();
        if swa {
            out.text(open);
            if !pre.is_empty() {
                out.text(&format!("{pre} "));
            }
            out.runs.extend(body.runs);
            if !post.is_empty() {
                out.text(&format!("{}{post}", s.cmt));
            }
            out.text(close);
        } else {
            out.runs.extend(body.runs);
            if !post.is_empty() {
                out.text(&format!("{}{post}", s.cmt));
            }
            if !date.is_empty() {
                out.text(close);
            }
        }
        merge(out)
    }
}

/// `(\NAT@swa, \NAT@par, \NAT@ctype)` of a natbib command. `\cite` is
/// parenthetical in numerical mode or with an optional argument.
fn natbib_flags(kind: CiteKind, cite_parenthetical: bool) -> (bool, bool, u8) {
    match kind {
        CiteKind::Cite => (cite_parenthetical, true, 0),
        CiteKind::Citet => (false, true, 0),
        CiteKind::Citep => (true, true, 0),
        CiteKind::Citealt => (false, false, 0),
        CiteKind::Citealp | CiteKind::Citenum => (true, false, 0),
        CiteKind::Citeauthor => (false, false, 1),
        CiteKind::Citeyear => (false, false, 2),
        CiteKind::Citeyearpar => (true, true, 2),
    }
}

/// natbib's `[pre][post]`: one optional argument is the post-note.
fn notes(optionals: &[String]) -> (String, String) {
    match optionals {
        [] => (String::new(), String::new()),
        [post] => (String::new(), post.trim().to_string()),
        [pre, post, ..] => (pre.trim().to_string(), post.trim().to_string()),
    }
}

/// Adjacent plain runs merged (the bold markers stay separate).
fn merge(out: Out) -> Out {
    let mut merged = Out::default();
    for (text, bold) in out.runs {
        if bold {
            merged.bold(&text);
        } else {
            merged.text(&text);
        }
    }
    merged
}

/// `\NAT@sort@cites@`: numerical keys in ascending order, then every key
/// without a number (undefined, or a non-numeric label) in citation order.
fn sort_keys(keys: &[String], bibliography: &Bibliography) -> Vec<String> {
    let mut numbered: Vec<(i64, &String)> = Vec::new();
    let mut rest: Vec<String> = Vec::new();
    for key in keys {
        match bibliography.item(key).and_then(|item| is_number(&item.natbib_num)) {
            Some(n) => numbered.push((n, key)),
            None => rest.push(key.clone()),
        }
    }
    numbered.sort_by_key(|(n, _)| *n);
    numbered.into_iter().map(|(_, key)| key.clone()).chain(rest).collect()
}

/// The kernel's `\@citex`/`\@cite`: `[<label>,\penalty\@m\ <label>, <note>]`.
/// An undefined key is `\hbox{\reset@font\bfseries ?}`.
fn kernel_cite(keys: &[String], note: Option<&str>, bibliography: &Bibliography) -> Out {
    let mut out = Out::default();
    out.text("[");
    for (index, key) in keys.iter().enumerate() {
        if index > 0 {
            out.runs.push((", ".into(), false));
        }
        match bibliography.resolve(key) {
            Some(label) => out.runs.push((label.to_string(), false)),
            None => out.bold("?"),
        }
    }
    if let Some(note) = note {
        // `~` is TeX's tie: an ordinary interword space that just does not
        // break a line; the compiler's own layout never breaks inside a
        // `\cite` note.
        out.runs.push((format!(", {}", note.replace('~', " ")), false));
    }
    out.runs.push(("]".into(), false));
    out
}

/// The kernel runs are already split; kept as a function so the split
/// stays in one place.
fn split_kernel_runs(text: &str) -> Vec<String> {
    vec![text.to_string()]
}

fn text_run(text: &str, span: Span, style: TextStyle, space_before: bool) -> Inline {
    Inline::Text {
        text: text.to_string(),
        span,
        style,
        space_before,
    }
}

/// The heading `thebibliography` opens with: article.cls `\section*{\refname}`,
/// report/book `\chapter*{\bibname}`, and natbib's `\bibsection` (the same,
/// or `\section*{\bibname}` with `sectionbib`).
pub fn bibliography_heading(class: Option<&str>, style: &CiteStyle) -> String {
    let chapters = matches!(class, Some("report" | "book"));
    let _ = style.sectionbib;
    if chapters { "Bibliography" } else { "References" }.to_string()
}

/// The `[<widest-label>]` bracket text a `\bibitem`'s own marker or
/// `thebibliography`'s `\labelwidth` measures, given the environment's
/// widest-label argument (`\begin{thebibliography}{99}`'s `"99"`).
pub fn label_bracket(text: &str) -> String {
    format!("[{text}]")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize_document;
    use crate::DocumentId;

    fn scan(source: &str) -> (Bibliography, Vec<Diagnostic>) {
        let tokens = tokenize_document(source, DocumentId::default());
        let mut diags = Vec::new();
        let bibliography = prescan(&tokens, &mut diags);
        (bibliography, diags)
    }

    fn cite_text(style: CiteStyle, command: &str, optionals: &[&str], keys: &str, bib: &Bibliography) -> String {
        let mut citer = Citer::new(style);
        let (kind, capital) = CiteKind::from_command(command.trim_end_matches('*')).unwrap();
        let request = CiteRequest {
            kind,
            capital,
            star: command.ends_with('*'),
            optionals: optionals.iter().map(|s| s.to_string()).collect(),
            keys: keys.split(',').map(|k| k.to_string()).collect(),
        };
        let mut diags = Vec::new();
        citer.cite(&request, command, bib, Span::new(0, 0), &mut diags).1.text
    }

    const NAT: &str = r"\begin{thebibliography}{4}
\bibitem[Knuth(1984)]{k84}A.
\bibitem[Knuth(1986)]{k86}B.
\bibitem[Knuth and Plass(1981)]{kp}C.
\bibitem[Hammer et~al.(2014)Hammer, Phang, Hicks, and Foster]{h}D.
\end{thebibliography}";

    #[test]
    fn numbers_plain_bibitems_in_order() {
        let (bib, diags) = scan(
            r"\begin{thebibliography}{9}\bibitem{a}First.\bibitem{b}Second.\end{thebibliography}",
        );
        assert!(diags.is_empty());
        assert_eq!(bib.resolve("a"), Some("1"));
        assert_eq!(bib.resolve("b"), Some("2"));
        assert_eq!(bib.label_at(0), Some("1"));
        assert_eq!(bib.label_at(1), Some("2"));
    }

    #[test]
    fn optional_label_overrides_the_number_without_consuming_one() {
        let (bib, _) = scan(
            r"\begin{thebibliography}{9}\bibitem[Knuth 1984]{tex}A.\bibitem{b}B.\end{thebibliography}",
        );
        assert_eq!(bib.resolve("tex"), Some("Knuth 1984"));
        // The un-labelled entry after it is still numbered 1: the labelled
        // entry did not consume a number, matching `\@lbibitem`.
        assert_eq!(bib.resolve("b"), Some("1"));
    }

    #[test]
    fn duplicate_key_warns_and_the_second_wins() {
        let (bib, diags) = scan(
            r"\begin{thebibliography}{9}\bibitem{a}First.\bibitem{a}Second.\end{thebibliography}",
        );
        assert_eq!(bib.resolve("a"), Some("2"));
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("duplicate \\bibitem{a}"));
    }

    #[test]
    fn bibitem_outside_thebibliography_is_not_registered() {
        let (bib, _) = scan(r"\bibitem{a}Stray.");
        assert_eq!(bib.resolve("a"), None);
    }

    #[test]
    fn kernel_cite_with_note_and_undefined_key() {
        let (bib, _) = scan(r"\begin{thebibliography}{9}\bibitem{a}A.\bibitem{b}B.\end{thebibliography}");
        assert_eq!(cite_text(CiteStyle::default(), "cite", &[], "a,b", &bib), "[1, 2]");
        assert_eq!(cite_text(CiteStyle::default(), "cite", &["p.~2"], "a", &bib), "[1, p. 2]");
        assert_eq!(cite_text(CiteStyle::default(), "cite", &[], "a,zz,b", &bib), "[1, ?, 2]");
    }

    #[test]
    fn natbib_author_year_citations_follow_natbib_sty() {
        let (bib, _) = scan(NAT);
        let ay = || CiteStyle::natbib("").0;
        assert_eq!(cite_text(ay(), "citet", &[], "k84", &bib), "Knuth (1984)");
        assert_eq!(cite_text(ay(), "citep", &[], "kp", &bib), "(Knuth and Plass, 1981)");
        assert_eq!(cite_text(ay(), "citep", &[], "k84,kp", &bib), "(Knuth, 1984; Knuth and Plass, 1981)");
        assert_eq!(cite_text(ay(), "citep", &[], "k84,k86", &bib), "(Knuth, 1984, 1986)");
        assert_eq!(cite_text(ay(), "citet", &[], "k84,kp", &bib), "Knuth (1984); Knuth and Plass (1981)");
        assert_eq!(cite_text(ay(), "citet", &[], "k84,k86", &bib), "Knuth (1984, 1986)");
        assert_eq!(cite_text(ay(), "citep", &["see", "p.~5"], "k84", &bib), "(see Knuth, 1984, p.~5)");
        assert_eq!(cite_text(ay(), "citet", &["chap.~2"], "k84", &bib), "Knuth (1984, chap.~2)");
        assert_eq!(cite_text(ay(), "citealt", &[], "k84", &bib), "Knuth 1984");
        assert_eq!(cite_text(ay(), "citealp", &[], "k84,kp", &bib), "Knuth, 1984; Knuth and Plass, 1981");
        assert_eq!(cite_text(ay(), "citeauthor", &[], "kp", &bib), "Knuth and Plass");
        assert_eq!(cite_text(ay(), "citeyear", &[], "kp", &bib), "1981");
        assert_eq!(cite_text(ay(), "citeyearpar", &[], "kp", &bib), "(1981)");
        assert_eq!(cite_text(ay(), "citet*", &[], "h", &bib), "Hammer, Phang, Hicks, and Foster (2014)");
        assert_eq!(cite_text(ay(), "citet", &[], "h", &bib), "Hammer et~al. (2014)");
        assert_eq!(cite_text(ay(), "cite", &[], "k84", &bib), "Knuth (1984)");
        assert_eq!(cite_text(ay(), "citep", &[], "k84,zz", &bib), "(Knuth, 1984; ?)");
    }

    #[test]
    fn natbib_numerical_citations_follow_natbib_sty() {
        let (bib, _) = scan(NAT);
        let num = |o: &str| CiteStyle::natbib(o).0;
        assert_eq!(cite_text(num("numbers"), "citep", &[], "k84,kp", &bib), "[1, 3]");
        assert_eq!(cite_text(num("numbers"), "cite", &[], "k84", &bib), "[1]");
        assert_eq!(cite_text(num("numbers"), "citet", &[], "k84", &bib), "Knuth [1]");
        assert_eq!(cite_text(num("numbers"), "citet", &[], "k84,k86", &bib), "Knuth [1, 2]");
        assert_eq!(cite_text(num("numbers"), "citet", &[], "k84,kp", &bib), "Knuth [1], Knuth and Plass [3]");
        assert_eq!(cite_text(num("numbers"), "citep", &["see", ""], "kp", &bib), "[see 3]");
        assert_eq!(cite_text(num("numbers"), "citep", &["p.~7"], "k84", &bib), "[1, p.~7]");
        assert_eq!(cite_text(num("numbers"), "citealp", &[], "h", &bib), "4");
        assert_eq!(cite_text(num("numbers,compress"), "cite", &[], "k84,k86,kp", &bib), "[1\u{2013}3]");
        assert_eq!(cite_text(num("numbers,compress"), "cite", &[], "k84,k86", &bib), "[1, 2]");
        assert_eq!(cite_text(num("numbers,compress"), "cite", &[], "k84,kp,h", &bib), "[1, 3, 4]");
        assert_eq!(cite_text(num("numbers,sort"), "cite", &[], "h,kp,k84", &bib), "[1, 3, 4]");
        assert_eq!(cite_text(num("numbers,sort&compress"), "cite", &[], "h,kp,k86", &bib), "[2\u{2013}4]");
        assert_eq!(cite_text(num("super"), "cite", &[], "k84,kp", &bib), "1;3");
        // pdflatex: `[1? , 3]` and `?]`.
        assert_eq!(cite_text(num("numbers"), "citep", &[], "k84,zz,kp", &bib), "[1? , 3]");
        assert_eq!(cite_text(num("numbers"), "citet", &[], "zz", &bib), "? ]");
    }

    #[test]
    fn bibpunct_and_setcitestyle_change_the_punctuation() {
        let (bib, _) = scan(NAT);
        let mut style = CiteStyle::natbib("").0;
        style.bibpunct(None, &["[", "]", ",", "a", ",", ","].map(String::from));
        assert_eq!(cite_text(style.clone(), "citep", &[], "k84,kp", &bib), "[Knuth, 1984, Knuth and Plass, 1981]");
        style.setcitestyle("round,semicolon,aysep={},notesep={: }");
        assert_eq!(cite_text(style.clone(), "citep", &[], "k84", &bib), "(Knuth 1984)");
        assert_eq!(cite_text(style, "citep", &["p.~3"], "kp", &bib), "(Knuth and Plass 1981: p.~3)");
        let mut plainnat = CiteStyle::natbib("").0;
        plainnat.named_style("plainnat", false);
        assert_eq!(cite_text(plainnat, "citep", &[], "k84,kp", &bib), "[Knuth, 1984, Knuth and Plass, 1981]");
    }

    #[test]
    fn author_year_with_a_plain_entry_settles_on_numbers() {
        let (bib, _) = scan(r"\begin{thebibliography}{9}\bibitem{a}A.\bibitem[Knuth(1984)]{b}B.\end{thebibliography}");
        let style = CiteStyle::natbib("").0;
        assert!(bib.forces_numbers(&style));
        assert_eq!(bib.list_label(1, &style).as_deref(), Some("[2]"));
        // `\NAT@force@numbers` only sets `\NAT@numberstrue`: the
        // author-year brackets and separator stay.
        assert_eq!(cite_text(style, "citep", &[], "a,b", &bib), "(1; 2)");
    }

    #[test]
    fn natexlab_labels_read_their_extra_letter() {
        let (bib, _) = scan(r"\begin{thebibliography}{9}\bibitem[Knuth(1984{\natexlab{a}})]{a}A.\bibitem[Knuth(1984{\natexlab{b}})]{b}B.\end{thebibliography}");
        let style = CiteStyle::natbib("").0;
        assert_eq!(cite_text(style, "citep", &[], "a,b", &bib), "(Knuth, 1984a,b)");
    }
}
