//! Theorem-like environments: the kernel's `\newtheorem` (no amsthm) and
//! amsthm's `\newtheorem`/`\newtheorem*`, `\theoremstyle`,
//! `\newtheoremstyle`, `\swapnumbers`, `proof`, `\qed` and `\qedhere`, plus
//! amsmath's `\numberwithin`. See `src/theorems.rs`: kernel heads are
//! `{\bfseries Name Number (Note)}` with no punctuation and an italic body;
//! amsthm `plain` bolds the head (period included) and italicises the body,
//! `definition` bolds the head and leaves the body upright, `remark`
//! italicises the head and leaves the body upright.

use flashtex_compiler::parser::{self, Block, Inline, TextStyle};
use flashtex_compiler::theorems::{
    Dimen, HeadSpace, QedPlacement, QedSymbol, TheoremKind, ThmSkip,
};

fn messages(source: &str) -> Vec<String> {
    parser::parse(source)
        .diagnostics
        .into_iter()
        .map(|d| d.message)
        .collect()
}

/// `source` with amsthm loaded first.
fn amsthm(source: &str) -> String {
    format!("\\usepackage{{amsthm}}\n{source}")
}

/// Every `Inline::Text` run across every paragraph and list item, in
/// document order, as `(text, style)` pairs.
fn text_runs(source: &str) -> Vec<(String, TextStyle)> {
    parser::parse(source)
        .blocks
        .into_iter()
        .flat_map(|block| match block {
            Block::Paragraph(inlines) => inlines,
            Block::ListItem { content, .. } => content,
            _ => Vec::new(),
        })
        .filter_map(|inline| match inline {
            Inline::Text { text, style, .. } => Some((text, style)),
            _ => None,
        })
        .collect()
}

fn plain_texts(source: &str) -> Vec<String> {
    text_runs(source)
        .into_iter()
        .map(|(text, _)| text)
        .collect()
}

const ITALIC: TextStyle = TextStyle {
    bold: false,
    italic: true,
    family: flashtex_compiler::parser::TextFamily::Roman,
    size: None,
};

#[test]
fn plain_style_bolds_head_and_period_and_italicises_body() {
    let source = amsthm(
        r"\newtheorem{theorem}{Theorem}
\begin{theorem}
Every prime greater than two is odd.
\end{theorem}",
    );
    let runs = text_runs(&source);
    let (head_text, head_style) = &runs[0];
    assert_eq!(head_text, "Theorem 1");
    assert_eq!(*head_style, TextStyle::BOLD);
    // amsthm sets `\the\thm@headpunct` inside the head font.
    assert_eq!(runs[1].0, ".");
    assert_eq!(runs[1].1, TextStyle::BOLD);
    let body: Vec<&String> = runs[2..].iter().map(|(text, _)| text).collect();
    assert!(body.contains(&&"Every".to_string()));
    for (_, style) in &runs[2..] {
        assert_eq!(*style, ITALIC, "body text must be italic in plain style");
    }
}

#[test]
fn kernel_newtheorem_has_a_bold_note_and_no_punctuation() {
    let source = r"\newtheorem{theorem}{Theorem}
\begin{theorem}[Euclid]
Primes never end.
\end{theorem}";
    let runs = text_runs(source);
    assert_eq!(runs[0], ("Theorem 1".to_string(), TextStyle::BOLD));
    assert_eq!(runs[1], ("(Euclid)".to_string(), TextStyle::BOLD));
    assert_eq!(runs[2], ("Primes".to_string(), ITALIC));
    let parsed = parser::parse(source);
    let record = &parsed.theorems[0];
    assert_eq!(record.kind, TheoremKind::Kernel);
    assert_eq!(record.head_space, HeadSpace::LabelSep);
    assert_eq!(record.head_punct, "");
    assert_eq!(record.number.as_deref(), Some("1"));
    assert_eq!(record.note.as_deref(), Some("Euclid"));
    let begin = &source[record.begin.start..record.begin.end];
    assert_eq!(begin, r"\begin{theorem}[Euclid]");
    let end = record.end.expect("closed");
    assert!(source[end.start..].starts_with(r"\end"));
}

#[test]
fn successive_theorems_number_sequentially() {
    let source = r"\newtheorem{theorem}{Theorem}
\begin{theorem}
First.
\end{theorem}
\begin{theorem}
Second.
\end{theorem}";
    let texts = plain_texts(source);
    assert!(texts.contains(&"Theorem 1".to_string()));
    assert!(texts.contains(&"Theorem 2".to_string()));
}

#[test]
fn optional_note_is_upright_medium_and_parenthesized() {
    let source = amsthm(
        r"\newtheorem{theorem}{Theorem}
\begin{theorem}[Fermat]
Statement.
\end{theorem}",
    );
    let runs = text_runs(&source);
    assert_eq!(runs[0].0, "Theorem 1");
    assert_eq!(runs[1].0, "(Fermat)");
    assert_eq!(
        runs[1].1,
        TextStyle::default(),
        "\\thm@notefont is \\fontseries\\mddefault\\upshape"
    );
    assert_eq!(runs[2], (".".to_string(), TextStyle::BOLD));
}

#[test]
fn theoremstyle_definition_keeps_body_upright() {
    let source = amsthm(
        r"\theoremstyle{definition}
\newtheorem{definition}{Definition}
\begin{definition}
A number is even if it is divisible by two.
\end{definition}",
    );
    let runs = text_runs(&source);
    assert_eq!(runs[0].0, "Definition 1");
    assert_eq!(
        runs[0].1,
        TextStyle::BOLD,
        "definition style still bolds the head"
    );
    for (_, style) in &runs[2..] {
        assert_eq!(
            *style,
            TextStyle::default(),
            "definition style leaves the body upright"
        );
    }
}

#[test]
fn theoremstyle_remark_italicises_head_and_halves_the_skips() {
    let source = amsthm(
        r"\theoremstyle{remark}
\newtheorem{remark}{Remark}
\begin{remark}
This generalizes to any ring.
\end{remark}",
    );
    let runs = text_runs(&source);
    assert_eq!(runs[0].0, "Remark 1");
    assert_eq!(runs[0].1, ITALIC, "remark style italicises the head");
    for (_, style) in &runs[2..] {
        assert_eq!(
            *style,
            TextStyle::default(),
            "remark style leaves the body upright"
        );
    }
    let record = &parser::parse(&source).theorems[0];
    assert_eq!(record.preskip, ThmSkip::Topsep { scale: 0.5 });
    assert_eq!(record.postskip, ThmSkip::Topsep { scale: 0.5 });
}

#[test]
fn theoremstyle_switches_back_and_forth() {
    let source = amsthm(
        r"\newtheorem{theorem}{Theorem}
\theoremstyle{definition}
\newtheorem{definition}{Definition}
\theoremstyle{plain}
\newtheorem{lemma}{Lemma}
\begin{theorem}
T.
\end{theorem}
\begin{definition}
D.
\end{definition}
\begin{lemma}
L.
\end{lemma}",
    );
    let runs = text_runs(&source);
    // Each body is a single one-word run after the head and its period.
    let body_of = |head: &str| -> TextStyle {
        runs.iter()
            .position(|(text, _)| text == head)
            .and_then(|i| runs.get(i + 2))
            .map(|(_, style)| *style)
            .unwrap()
    };
    assert_eq!(body_of("Theorem 1"), ITALIC);
    assert_eq!(body_of("Definition 1"), TextStyle::default());
    assert_eq!(body_of("Lemma 1"), ITALIC);
}

#[test]
fn starred_newtheorem_is_unnumbered_and_does_not_advance_any_counter() {
    let source = amsthm(
        r"\newtheorem*{remarkstar}{Remark}
\begin{remarkstar}
No number here.
\end{remarkstar}
\begin{remarkstar}
Still no number.
\end{remarkstar}",
    );
    let texts = plain_texts(&source);
    assert!(texts.contains(&"Remark".to_string()));
    assert!(!texts.iter().any(|t| t.starts_with("Remark ")));
}

#[test]
fn shared_counter_interleaves_with_the_theorem_it_shares() {
    let source = r"\newtheorem{theorem}{Theorem}
\newtheorem{lemma}[theorem]{Lemma}
\begin{theorem}
T1.
\end{theorem}
\begin{lemma}
L1.
\end{lemma}
\begin{theorem}
T2.
\end{theorem}";
    let texts = plain_texts(source);
    assert!(texts.contains(&"Theorem 1".to_string()));
    assert!(texts.contains(&"Lemma 2".to_string()));
    assert!(texts.contains(&"Theorem 3".to_string()));
}

#[test]
fn undefined_shared_counter_is_an_honest_error() {
    let source = r"\newtheorem{lemma}[undefinedname]{Lemma}";
    let msgs = messages(source);
    assert!(
        msgs.iter()
            .any(|m| m.contains("undefinedname") && m.contains("undefined")),
        "{msgs:?}"
    );
}

#[test]
fn within_section_resets_per_section_and_prints_section_dot_number() {
    let source = r"\documentclass{article}
\newtheorem{theorem}{Theorem}[section]
\begin{document}
\section{One}
\begin{theorem}
A.
\end{theorem}
\begin{theorem}
B.
\end{theorem}
\section{Two}
\begin{theorem}
C.
\end{theorem}
\end{document}";
    let texts = plain_texts(source);
    assert!(texts.contains(&"Theorem 1.1".to_string()), "{texts:?}");
    assert!(texts.contains(&"Theorem 1.2".to_string()), "{texts:?}");
    assert!(texts.contains(&"Theorem 2.1".to_string()), "{texts:?}");
}

#[test]
fn within_another_theorem_counter_and_numberwithin() {
    let source = r"\documentclass{article}
\usepackage{amsmath,amsthm}
\newtheorem{theorem}{Theorem}
\numberwithin{theorem}{section}
\newtheorem{corollary}{Corollary}[theorem]
\begin{document}
\section{One}
\begin{theorem}
A.
\end{theorem}
\begin{corollary}
B.
\end{corollary}
\begin{theorem}
C.
\end{theorem}
\begin{corollary}
D.
\end{corollary}
\end{document}";
    let texts = plain_texts(source);
    for expected in [
        "Theorem 1.1",
        "Corollary 1.1.1",
        "Theorem 1.2",
        "Corollary 1.2.1",
    ] {
        assert!(
            texts.contains(&expected.to_string()),
            "{expected}: {texts:?}"
        );
    }
}

#[test]
fn unsupported_within_counter_warns_and_falls_back_to_a_plain_counter() {
    let source = r"\newtheorem{theorem}{Theorem}[chapter]
\begin{theorem}
A.
\end{theorem}
\begin{theorem}
B.
\end{theorem}";
    let msgs = messages(source);
    assert!(
        msgs.iter()
            .any(|m| m.contains("chapter") && m.contains("recognised but not implemented")),
        "{msgs:?}"
    );
    let texts = plain_texts(source);
    assert!(texts.contains(&"Theorem 1".to_string()));
    assert!(texts.contains(&"Theorem 2".to_string()));
}

#[test]
fn unknown_theoremstyle_name_is_an_honest_error() {
    let msgs = messages(r"\theoremstyle{fancy}");
    assert!(msgs.iter().any(|m| m.contains("fancy")), "{msgs:?}");
}

#[test]
fn swapnumbers_puts_the_number_first() {
    let source = amsthm(
        r"\swapnumbers
\newtheorem{theorem}{Theorem}
\begin{theorem}
A.
\end{theorem}",
    );
    assert_eq!(text_runs(&source)[0].0, "1 Theorem");
    assert!(parser::parse(&source).theorems[0].swap);
}

#[test]
fn newtheoremstyle_records_every_parameter() {
    let source = amsthm(
        r"\newtheoremstyle{note}{3pt}{6pt plus 2pt}{\itshape}{2em}{\bfseries}{:}{.5em}{}
\newtheoremstyle{break}{}{}{}{}{\itshape}{.}{\newline}{}
\newtheoremstyle{spaced}{\topsep}{\topsep}{}{}{\bfseries}{.}{ }{}
\theoremstyle{note}
\newtheorem{claim}{Claim}
\theoremstyle{break}
\newtheorem{fact}{Fact}
\theoremstyle{spaced}
\newtheorem{prop}{Proposition}
\begin{claim}
A.
\end{claim}
\begin{fact}
B.
\end{fact}
\begin{prop}
C.
\end{prop}",
    );
    let parsed = parser::parse(&source);
    assert!(
        parsed
            .diagnostics
            .iter()
            .all(|d| !d.message.contains("newtheoremstyle")),
        "{:?}",
        parsed.diagnostics
    );
    let [claim, fact, prop] = &parsed.theorems[..] else {
        panic!("{:?}", parsed.theorems)
    };
    assert_eq!(
        claim.preskip,
        ThmSkip::Glue {
            pt: 3.0,
            plus: 0.0,
            minus: 0.0
        }
    );
    assert_eq!(
        claim.postskip,
        ThmSkip::Glue {
            pt: 6.0,
            plus: 2.0,
            minus: 0.0
        }
    );
    assert_eq!(claim.body_font, ITALIC);
    assert_eq!(claim.indent, Some(Dimen::Em(2.0)));
    assert_eq!(claim.head_font, TextStyle::BOLD);
    assert_eq!(claim.head_punct, ":");
    assert_eq!(
        claim.head_space,
        HeadSpace::Glue {
            pt: 5.0,
            plus: 0.0,
            minus: 0.0
        }
    );
    assert_eq!(fact.preskip, ThmSkip::Topsep { scale: 1.0 });
    assert_eq!(fact.head_space, HeadSpace::Newline);
    assert_eq!(fact.head_font, ITALIC);
    assert_eq!(fact.indent, None);
    assert_eq!(prop.head_space, HeadSpace::Space);
    assert_eq!(prop.body_font, TextStyle::default());
    let runs = text_runs(&source);
    assert!(
        runs.contains(&(":".to_string(), TextStyle::BOLD)),
        "{runs:?}"
    );
}

#[test]
fn newtheoremstyle_custom_head_specification_is_diagnosed() {
    let msgs = messages(&amsthm(
        r"\newtheoremstyle{custom}{}{}{}{}{\bfseries}{.}{ }{\thmname{#1}}",
    ));
    assert!(
        msgs.iter()
            .any(|m| m.contains("custom head specification") && m.contains("not implemented")),
        "{msgs:?}"
    );
}

#[test]
fn proof_has_italic_head_and_right_flushed_qed_symbol() {
    let source = r"\begin{proof}
This follows directly.
\end{proof}";
    let runs = text_runs(source);
    assert_eq!(runs[0].0, "Proof.");
    assert_eq!(runs[0].1, ITALIC);
    for (text, style) in &runs[1..runs.len() - 1] {
        if text != "∎" {
            assert_eq!(*style, TextStyle::default(), "proof body must be upright");
        }
    }
    let (last_text, last_style) = runs.last().unwrap();
    assert_eq!(last_text, "∎");
    assert_eq!(*last_style, TextStyle::default());

    let parsed = parser::parse(source);
    let has_hfill_before_qed = parsed.blocks.iter().any(|block| {
        if let Block::Paragraph(inlines) = block {
            inlines.windows(2).any(|pair| {
                matches!(pair[0], Inline::HFill { .. })
                    && matches!(&pair[1], Inline::Text { text, .. } if text == "∎")
            })
        } else {
            false
        }
    });
    assert!(has_hfill_before_qed, "{:?}", parsed.blocks);
    let record = &parsed.theorems[0];
    assert_eq!(record.kind, TheoremKind::Proof);
    assert_eq!(record.head_space, HeadSpace::LabelSep);
    assert_eq!(
        record.preskip,
        ThmSkip::Glue {
            pt: 6.0,
            plus: 6.0,
            minus: 0.0
        }
    );
    let [mark] = &parsed.qed_marks[..] else {
        panic!("{:?}", parsed.qed_marks)
    };
    assert_eq!(mark.placement, QedPlacement::EndOfProof);
    assert_eq!(mark.symbol, QedSymbol::OpenBox);
    assert_eq!(Some(mark.span), record.end);
}

#[test]
fn proof_custom_heading_replaces_default_but_keeps_the_period() {
    let source = r"\begin{proof}[Proof of Lemma 2]
Body.
\end{proof}";
    let runs = text_runs(source);
    assert_eq!(runs[0].0, "Proof of Lemma 2.");
    assert_eq!(runs[0].1, ITALIC);
    // `\@addpunct{.}` adds nothing after a sentence-ending mark.
    assert_eq!(
        text_runs("\\begin{proof}[Why?]\nBody.\n\\end{proof}")[0].0,
        "Why?"
    );
}

#[test]
fn renewed_proofname_is_the_default_heading() {
    let source = amsthm(
        r"\renewcommand{\proofname}{Beweis}
\begin{proof}
Kurz.
\end{proof}",
    );
    let parsed = parser::parse(&source);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert_eq!(text_runs(&source)[0].0, "Beweis.");
}

#[test]
fn qedhere_in_a_list_item_places_the_symbol_there_and_not_at_the_end() {
    let source = amsthm(
        r"\begin{proof}
Two cases.
\begin{itemize}
\item Even.
\item Odd. \qedhere
\end{itemize}
\end{proof}
After.",
    );
    let parsed = parser::parse(&source);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let [mark] = &parsed.qed_marks[..] else {
        panic!("{:?}", parsed.qed_marks)
    };
    assert_eq!(mark.placement, QedPlacement::Here);
    assert!(source[mark.span.start..].starts_with(r"\qedhere"));
    let qeds = text_runs(&source)
        .iter()
        .filter(|(text, _)| text == "∎")
        .count();
    assert_eq!(qeds, 1);
    let in_item = parsed.blocks.iter().any(|block| {
        matches!(block, Block::ListItem { content, .. }
            if content.iter().any(|i| matches!(i, Inline::Text { text, .. } if text == "∎")))
    });
    assert!(in_item, "{:?}", parsed.blocks);
}

#[test]
fn qedhere_in_a_display_is_recorded_for_the_display() {
    let source = r"\documentclass{article}
\usepackage{amsmath,amsthm}
\begin{document}
\begin{proof}
Hence
\begin{equation*}
  a = b. \qedhere
\end{equation*}
\end{proof}
\end{document}";
    let parsed = parser::parse(source);
    assert!(
        parsed
            .diagnostics
            .iter()
            .all(|d| !d.message.contains("qedhere")),
        "{:?}",
        parsed.diagnostics
    );
    let [mark] = &parsed.qed_marks[..] else {
        panic!("{:?}", parsed.qed_marks)
    };
    assert_eq!(mark.placement, QedPlacement::Display);
    assert!(source[mark.span.start..].starts_with(r"\qedhere"));
    assert!(!text_runs(source).iter().any(|(text, _)| text == "∎"));
}

#[test]
fn explicit_qed_is_a_mark_too() {
    let parsed = parser::parse("Done. \\qed");
    let [mark] = &parsed.qed_marks[..] else {
        panic!("{:?}", parsed.qed_marks)
    };
    assert_eq!(mark.placement, QedPlacement::Command);
}

#[test]
fn amsthm_alone_is_silent() {
    let msgs =
        messages(r"\documentclass{article}\usepackage{amsthm}\begin{document}x\end{document}");
    assert!(msgs.is_empty(), "{msgs:?}");
}

#[test]
fn amsmath_and_amssymb_still_warn_once_amsthm_no_longer_does() {
    let msgs = messages(
        r"\documentclass{article}\usepackage{amsmath,amssymb,amsthm}\begin{document}x\end{document}",
    );
    let package_msgs: Vec<&String> = msgs
        .iter()
        .filter(|m| m.contains("recognised but not implemented"))
        .collect();
    assert_eq!(package_msgs.len(), 1, "{msgs:?}");
    assert!(package_msgs[0].contains("amsmath"));
    assert!(package_msgs[0].contains("amssymb"));
    assert!(!package_msgs[0].contains("amsthm"));
}

#[test]
fn unregistered_environment_name_still_reports_the_generic_gap() {
    // A name that was never `\newtheorem`-declared is not silently treated
    // as a theorem: it still gets the honest generic diagnostic.
    let msgs = messages(r"\begin{claim}Not declared.\end{claim}");
    assert!(
        msgs.iter()
            .any(|m| m.contains("claim") && m.contains("not implemented")),
        "{msgs:?}"
    );
}
