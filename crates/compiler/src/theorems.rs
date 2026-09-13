//! Theorem-like environments: the LaTeX kernel's `\newtheorem`
//! (latex.ltx, ltthm.dtx `\@begintheorem`/`\@opargbegintheorem`), amsthm's
//! `\newtheorem`/`\newtheorem*`, `\theoremstyle`, `\newtheoremstyle`,
//! `\swapnumbers` and `proof` with `\qed`/`\qedhere` (amsthm.sty v2.20.6).
//!
//! The parser emits each head as ordinary inlines (so every consumer can
//! read the text) and records every instance as a [`TheoremRecord`] with
//! the structure a faithful layout needs: the kind of head (a kernel
//! `\trivlist` label or amsthm's `\deferred@thm@head`), its fonts,
//! punctuation, head space and indent, the `\thm@preskip`/`\thm@postskip`
//! glue, and the spans of `\begin` (with the optional note) and `\end`.
//! Every end-of-proof symbol the parser places is a [`QedMark`].
//!
//! Numbering uses the document's `xref::Counters`: `\newtheorem{env}{Name}`
//! defines a counter, `[shared]` reuses one, `[within]` resets with
//! `within` and prints `\the<within>.<n>` (`\@xnthm` / amsthm `\@xthm`), and
//! amsmath's `\numberwithin` changes an existing one.

use std::collections::HashMap;

use crate::parser::TextStyle;
use crate::Span;

/// Which definition of `\newtheorem` made the environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheoremKind {
    /// latex.ltx without amsthm: `\trivlist \item[\hskip\labelsep{\bfseries
    /// Name Number (Note)}]\itshape`. `\@topsepadd` is `\topsep`, plus
    /// `\partopsep` when `\begin` is read in vertical mode; `\endtrivlist`
    /// leaves `\@endpetrue` (a paragraph right after `\end` is not
    /// indented).
    Kernel,
    /// amsthm's `\@thm`: `\@topsep\thm@preskip`, `\@topsepadd\thm@postskip`,
    /// the head in `\@labels` unboxed into the first paragraph, and
    /// `\@endpefalse` at `\end`.
    Amsthm,
    /// amsthm's `proof`: `\par \normalfont \topsep6\p@\@plus6\p@ \trivlist
    /// \item[\hskip\labelsep\itshape Name\@addpunct{.}]` (always vertical
    /// mode, so `\partopsep` is added before and after), `\@endpefalse`.
    Proof,
}

/// `\thm@preskip`/`\thm@postskip`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThmSkip {
    /// `\topsep` of the list level where the environment begins, times
    /// `scale` (`1` for `plain`/`definition`, `1/2` for `remark`, amsthm.sty
    /// 232). Stretch and shrink scale with it.
    Topsep { scale: f64 },
    /// Explicit glue in points (`em`/`ex` at the class body size, where
    /// `\newtheoremstyle` evaluates them).
    Glue { pt: f64, plus: f64, minus: f64 },
}

/// A length whose unit a layout resolves against a font.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dimen {
    Pt(f64),
    /// `\newtheoremstyle`'s indent is read inside the head font, so `em` is
    /// that font's quad.
    Em(f64),
    Ex(f64),
}

/// What separates the head from the body.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HeadSpace {
    /// Kernel `\@item` and `proof`: the label box ends with `\hskip\labelsep`
    /// (rigid, inside the box), then `\penalty\z@`.
    LabelSep,
    /// amsthm `\hskip\thm@headsep` inside the unboxed head (default `5pt
    /// plus 1pt minus 1pt`, amsthm.sty 178): breakable, stretchable glue.
    Glue { pt: f64, plus: f64, minus: f64 },
    /// `\newtheoremstyle` head space `{ }`: `\fontdimen2` of the head font.
    Space,
    /// `\newtheoremstyle` head space `{\newline}`: `\thm@headsep` is zero and
    /// `\thmheadnl` breaks the line after the head.
    Newline,
}

/// One amsthm style (`\th@plain`, `\th@definition`, `\th@remark`, or a
/// `\newtheoremstyle`), or the kernel's fixed head.
#[derive(Debug, Clone, PartialEq)]
pub struct TheoremStyleSpec {
    pub preskip: ThmSkip,
    pub postskip: ThmSkip,
    /// `\normalfont` then the style's body font.
    pub body_font: TextStyle,
    /// `\thm@headfont` (`\bfseries` by default).
    pub head_font: TextStyle,
    /// `\thm@notefont` inside the head font: `\fontseries\mddefault\upshape`.
    pub note_font: TextStyle,
    /// `\thm@indent`: `None` is `\noindent`; `Some` is `\noindent\hbox
    /// to<indent>{}` (a `\newtheoremstyle` with a non-zero indent).
    pub indent: Option<Dimen>,
    /// `\thm@headpunct` (`.` by default), set in the head font.
    pub head_punct: String,
    pub head_space: HeadSpace,
}

const ITALIC: TextStyle = TextStyle {
    italic: true,
    bold: false,
    family: crate::parser::TextFamily::Roman,
    size: None,
};

impl TheoremStyleSpec {
    /// `\th@plain`: bold head, italic body, `\topsep` above and below.
    pub fn plain() -> Self {
        TheoremStyleSpec {
            preskip: ThmSkip::Topsep { scale: 1.0 },
            postskip: ThmSkip::Topsep { scale: 1.0 },
            body_font: ITALIC,
            head_font: TextStyle::BOLD,
            note_font: TextStyle::default(),
            indent: None,
            head_punct: ".".to_string(),
            head_space: HeadSpace::Glue {
                pt: 5.0,
                plus: 1.0,
                minus: 1.0,
            },
        }
    }

    /// `\th@definition`: bold head, upright body.
    pub fn definition() -> Self {
        TheoremStyleSpec {
            body_font: TextStyle::default(),
            ..Self::plain()
        }
    }

    /// `\th@remark`: italic head, upright body, half `\topsep` above and
    /// below.
    pub fn remark() -> Self {
        TheoremStyleSpec {
            preskip: ThmSkip::Topsep { scale: 0.5 },
            postskip: ThmSkip::Topsep { scale: 0.5 },
            body_font: TextStyle::default(),
            head_font: ITALIC,
            ..Self::plain()
        }
    }

    /// latex.ltx `\@begintheorem`: `{\bfseries Name Number (Note)}` (the note
    /// bold too), no punctuation, `\itshape` body.
    pub fn kernel() -> Self {
        TheoremStyleSpec {
            preskip: ThmSkip::Topsep { scale: 1.0 },
            postskip: ThmSkip::Topsep { scale: 1.0 },
            body_font: ITALIC,
            head_font: TextStyle::BOLD,
            note_font: TextStyle::BOLD,
            indent: None,
            head_punct: String::new(),
            head_space: HeadSpace::LabelSep,
        }
    }
}

/// amsthm's predefined styles, keyed by `\theoremstyle` name.
pub(crate) fn builtin_styles() -> HashMap<String, TheoremStyleSpec> {
    HashMap::from([
        ("plain".to_string(), TheoremStyleSpec::plain()),
        ("definition".to_string(), TheoremStyleSpec::definition()),
        ("remark".to_string(), TheoremStyleSpec::remark()),
    ])
}

/// One `\newtheorem` registration.
#[derive(Debug, Clone)]
pub struct TheoremDef {
    pub title: String,
    /// The style in force at `\newtheorem` (amsthm captures it there).
    pub style: TheoremStyleSpec,
    pub kind: TheoremKind,
    /// `\newtheorem*` defines an unnumbered environment.
    pub numbered: bool,
    /// The counter this environment steps (its own name, or the shared one).
    pub counter: String,
    /// amsthm `\swapnumbers` was in force at `\newtheorem`: `Number Name`.
    pub swap: bool,
}

/// One theorem-like environment instance, in `\begin` order.
#[derive(Debug, Clone, PartialEq)]
pub struct TheoremRecord {
    /// The environment name (`theorem`, `proof`, ...).
    pub environment: String,
    pub kind: TheoremKind,
    /// `\begin{env}` through the optional `[note]`. The head inlines the
    /// parser emits carry spans inside it.
    pub begin: Span,
    /// The `\end` command; `None` while the environment is unterminated.
    pub end: Option<Span>,
    /// `Theorem`, or the proof's heading (`\proofname` or the optional
    /// argument) without its punctuation.
    pub name: String,
    /// `\the<counter>` after `\refstepcounter`; `None` when unnumbered.
    pub number: Option<String>,
    /// The optional argument's text (without parentheses).
    pub note: Option<String>,
    /// `Number Name` (amsthm `\swappedhead`, a `~` between them).
    pub swap: bool,
    pub head_font: TextStyle,
    pub note_font: TextStyle,
    pub body_font: TextStyle,
    pub head_punct: String,
    pub head_space: HeadSpace,
    pub indent: Option<Dimen>,
    pub preskip: ThmSkip,
    pub postskip: ThmSkip,
}

/// Where an end-of-proof symbol was placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QedPlacement {
    /// `\end{proof}`'s `\popQED`: at the end of the proof's last paragraph.
    EndOfProof,
    /// `\qedhere` in text (a list item, say): at that point.
    Here,
    /// `\qedhere` inside a display: amsthm puts the symbol where the
    /// equation number goes (`\displaymath@qed`/`\equation@qed`); the span is
    /// the `\qedhere` command inside the display's span, and the parser
    /// emits no inline for it.
    Display,
    /// An explicit `\qed` in text.
    Command,
}

/// Which `\qedsymbol` is in force.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QedSymbol {
    /// amsthm's `\openbox`: `\hbox to.77778em{\hfil\vrule \vbox to.675em
    /// {\hrule width.6em\vfil\hrule}\vrule\hfil}` (0.4pt rules).
    OpenBox,
    /// A `\renewcommand{\qedsymbol}`; the parser still emits U+220E.
    Custom,
}

/// An end-of-proof symbol: amsthm's `\qed` is `\leavevmode\unskip
/// \penalty9999 \hbox{}\nobreak\hfill\quad\hbox{\qedsymbol}` (amsthm.sty
/// 273-279). Outside a display the parser emits `Inline::HFill` and the text
/// `∎`, both with this span.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QedMark {
    pub span: Span,
    pub placement: QedPlacement,
    pub symbol: QedSymbol,
}

/// An open `proof`: where its tokens start (to find a `\qedhere` read in
/// math mode) and whether a text `\qedhere` already placed the symbol.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ProofFrame {
    pub(crate) token_index: usize,
    pub(crate) qed_placed: bool,
}

/// `\newtheoremstyle`'s space above/below: empty or `\topsep` is
/// `\topsep`; otherwise `<dimen> [plus <dimen>] [minus <dimen>]`.
pub fn parse_skip(text: &str, body_pt: f64) -> Option<ThmSkip> {
    let text = text.trim();
    // `token_text` spells a control word without its backslash.
    if text.is_empty() || text == "\\topsep" || text == "topsep" {
        return Some(ThmSkip::Topsep { scale: 1.0 });
    }
    let (natural, rest) = match text.find("plus") {
        Some(at) => (&text[..at], Some(&text[at + 4..])),
        None => match text.find("minus") {
            Some(at) => (&text[..at], Some(&text[at..])),
            None => (text, None),
        },
    };
    let pt = crate::parser::parse_dimen_pt_at(natural, body_pt)?;
    let (mut plus, mut minus) = (0.0, 0.0);
    if let Some(rest) = rest {
        let (stretch, shrink) = match rest.find("minus") {
            Some(at) => (&rest[..at], Some(&rest[at + 5..])),
            None => (rest, None),
        };
        if !stretch.trim().is_empty() {
            plus = crate::parser::parse_dimen_pt_at(stretch, body_pt)?;
        }
        if let Some(shrink) = shrink {
            minus = crate::parser::parse_dimen_pt_at(shrink, body_pt)?;
        }
    }
    Some(ThmSkip::Glue { pt, plus, minus })
}

/// `\newtheoremstyle`'s indent: `em`/`ex` stay relative to the head font.
pub fn parse_dimen(text: &str) -> Option<Dimen> {
    let text = text.trim();
    if let Some(value) = text.strip_suffix("em") {
        return value.trim().parse().ok().map(Dimen::Em);
    }
    if let Some(value) = text.strip_suffix("ex") {
        return value.trim().parse().ok().map(Dimen::Ex);
    }
    crate::parser::parse_dimen_pt_at(text, 10.0).map(Dimen::Pt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_parse_topsep_and_glue() {
        assert_eq!(parse_skip("", 10.0), Some(ThmSkip::Topsep { scale: 1.0 }));
        assert_eq!(
            parse_skip("\\topsep", 10.0),
            Some(ThmSkip::Topsep { scale: 1.0 })
        );
        assert_eq!(
            parse_skip("3pt", 10.0),
            Some(ThmSkip::Glue {
                pt: 3.0,
                plus: 0.0,
                minus: 0.0
            })
        );
        assert_eq!(
            parse_skip("6pt plus 2pt minus 1pt", 10.0),
            Some(ThmSkip::Glue {
                pt: 6.0,
                plus: 2.0,
                minus: 1.0
            })
        );
        assert_eq!(parse_skip("\\baselineskip", 10.0), None);
        assert_eq!(parse_dimen("2em"), Some(Dimen::Em(2.0)));
        assert_eq!(parse_dimen("12pt"), Some(Dimen::Pt(12.0)));
    }
}
