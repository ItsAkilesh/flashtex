//! The skip at a `\trivlist`-to-`\trivlist` boundary that is not two lists.
//!
//! #712 fixed the list-to-list case: `\end{<list>}` is `\endtrivlist` ->
//! `\@endparenv`'s `\addvspace\@topsepadd`, the `\begin{<list>}` beside it is
//! `\@trivlist`'s `\addvspace\@topsep`, and two `\addvspace`s keep the larger
//! natural skip rather than summing — while `\@endparenv`'s `\par` leaves TeX
//! in vertical mode, so the second `\begin` takes `\partopsep` with no blank
//! line between them.
//!
//! Neither half is about *lists*. article.cls builds `center`, `flushleft` and
//! `flushright` as `\trivlist \centering \item\relax`, `quote`, `quotation` and
//! `verse` as `\list{}{...}\item\relax`, and amsthm builds every theorem as a
//! `\trivlist`. All of them end in the same `\endtrivlist`. Measured against
//! pdflatex, three things were wrong at those boundaries:
//!
//! * `\end{<list>}` then `\begin{center|quote|quotation|verse|<theorem>}`
//!   summed the list's closing `\@topsepadd` onto the environment's opening
//!   `\@topsep`: 10 pt + 8 pt = 18 pt where pdflatex puts max(10, 10) = 10 pt,
//!   i.e. **+7.971 bp** at every such boundary;
//! * `\end{<theorem>}` then any `\trivlist` environment, and `\end{quote}`
//!   then `\begin{quote}` (and every other pair the pipeline reads as one
//!   `ParaStyle` run), dropped `\partopsep`: **-1.992 bp**;
//! * both errors accumulated. Six `center`s in a row drifted -1.992, -3.985,
//!   -5.978, -7.970, -9.963 bp; alternating `center`/`itemize` drifted
//!   +5.978, +3.985, +9.963, +7.970 bp.
//!
//! An amsthm theorem is the one case that legitimately steps by `\topsep`
//! alone: `\@thm` assigns `\@topsep`/`\@topsepadd` from `\thm@preskip`/
//! `\thm@postskip`, so it never picks up `\partopsep`. Two adjacent theorems
//! are 19.925 bp apart, and that is pinned below so a fix for the others does
//! not sweep it up.
//!
//! ## The two gaps #722 left, closed here by measurement
//!
//! #722 stopped where its measurements stopped, and said so: `verbatim` and
//! `abstract` are `\trivlist`-derived too but were never measured, and only
//! the 10 pt base size was swept. Both are now measured, over 918 probe
//! documents. Two of the three guesses one would have made were right and
//! one was wrong, which is why they were measured rather than assumed:
//!
//! * **`verbatim` diverged.** `\@verbatim` is `\trivlist \item\relax ...`
//!   and `\endverbatim` is `\endtrivlist`, so it belongs in the same set —
//!   but only the pairs the pipeline reads as one `ParaStyle` run were
//!   wrong, `verbatim`/`verbatim` and `verbatim`/`flushleft`, each short by
//!   exactly one `\partopsep`: **-1.992 bp** at 10 pt, **-2.989** at 11 and
//!   12 pt. Six `verbatim`s in a row drifted to -9.963 bp at 10 pt.
//!   `verbatim` against `center`, `quote`, `verse`, `quotation`,
//!   `flushright`, a theorem or any list was already right.
//! * **`abstract` diverged, on both sides and for two different reasons.**
//!   `\end{abstract}` is `\endquotation` -> `\endlist` -> `\endtrivlist`,
//!   and what followed it was short by one `\partopsep` (-1.992/-2.989/
//!   -0.996 bp at 10/11/12 pt). In the other direction the pipeline builds
//!   the `\small` `\abstractname` head as a block inserted in front of the
//!   body, and the boundary skip the compiler had hung on that body stayed
//!   there, firing a second time below the head: `\end{itemize}` then
//!   `\begin{abstract}` was **+5.305/+5.729/+6.273 bp** long.
//! * **No boundary diverged only at 11 pt or 12 pt.** Every one that was
//!   wrong there was wrong at 10 pt too, and the two fixes above are the
//!   whole of it at all three sizes — so the size sweep found no third bug.
//!   It did find that one boundary is not size-invariant at all, which a
//!   10 pt-only sweep would have pinned as a single rule:
//!   `\end{abstract}\begin{thm}` is `\topsep`-only at 10 pt and 11 pt but
//!   26.401 bp at 12 pt, because which of the two competing `\addvspace`s
//!   is larger changes with the class option ([`Size::abstract_to_thm`]).
//!
//! ## Known gap, not a skip
//!
//! A `\begin{abstract}` on the line *directly* after body text, with no
//! blank line, is not recognised at all: the compiler never breaks the
//! paragraph there, so it reports one run of text across the `\begin`, the
//! pipeline finds no block inside the body to restyle, and no
//! `\abstractname` head is set (the body lands 32.553/36.115/42.744 bp
//! high, on the previous line). That is a compiler-side paragraph-splitting
//! gap, not a boundary skip — `\begin{center}` in the same position does
//! break the paragraph — and fixing it needs a change under `vendor/` and a
//! re-pin, which this line of work has deliberately stayed out of. It is
//! the only divergence left in the 918-document sweep.
//!
//! One more was measured outside that sweep and left alone because it is
//! unchanged by this work: in the `\if@twocolumn` branch, where the
//! `abstract` is a `\section*` and ordinary paragraphs rather than a
//! `quotation`, an `abstract` after a *list* sets its body 3.985/4.483/4.981
//! bp low at 10/11/12 pt. The `\abstractname` head itself is exact, the
//! `center`-before and standalone cases are exact, and base and branch print
//! byte-identical baselines for all nine two-column probes. This file sweeps
//! the one-column branch, which is what `\trivlist` boundaries are about.
//!
//! ## Oracle
//!
//! pdfTeX 1.40 (MacTeX 2026), T1 Latin Modern, `\pagestyle{empty}`, PyMuPDF
//! glyph origins, in bp, at the 10 pt, 11 pt and 12 pt `article` base sizes.
//! Every probe document below was run through pdflatex; the constants are
//! its measured baselines. pdflatex is an oracle only and never runs in the
//! product path.

mod common;

/// Word x/baseline gate for this project.
const WORD_TOL_BP: f64 = 0.5;

fn head(size: &str) -> String {
    format!(
        "\\documentclass[{size}]{{article}}\n\\usepackage[T1]{{fontenc}}\n\\usepackage{{lmodern}}\n\
         \\usepackage{{amsthm}}\n\\newtheorem{{thm}}{{Theorem}}\n\\pagestyle{{empty}}\n\\begin{{document}}\n"
    )
}

/// One class base size and the four pdflatex baselines this file checks at
/// it.
///
/// Sweeping all three is not a formality. `\topsep` and `\partopsep` are
/// assigned by `\@listI`, which `\@ptsize` selects (`size1?.clo`), so they
/// change with the class option: `\topsep` is 8/9/10 pt and `\partopsep`
/// 2/3/3 pt at 10/11/12 pt, on top of a `\baselineskip` of 12/13.6/14.5 pt.
/// A boundary rule that happens to come out right at 10 pt can be wrong at
/// the other two, and #722 measured only 10 pt.
struct Size {
    /// The `\documentclass` option.
    opt: &'static str,
    /// pdflatex's first baseline on an empty `article` page.
    first: f64,
    /// The step from one `\trivlist` environment to the next one beside it:
    /// `\baselineskip` + max(`\topsep` + `\partopsep`, the same).
    adjacent: f64,
    /// The step when only `\topsep` applies (`\baselineskip` + `\topsep`): a
    /// `\begin` read in horizontal mode, or an amsthm theorem, which sets
    /// `\@topsep`/`\@topsepadd` itself and so never takes `\partopsep`.
    topsep_only: f64,
    /// The step into an `abstract` *body*, which is a whole shape rather
    /// than one skip: the boundary's `\addvspace`, the `\small` centred
    /// `\abstractname` line, its `\vspace{-.5em}`, and the `quotation`'s own
    /// `\topsep` at `\small`.
    into_abstract: f64,
    /// The `abstract` body's own baseline below its `\abstractname` head,
    /// which is what an `abstract` at the very top of a page shows: no
    /// boundary `\addvspace` precedes it, so it is `into_abstract` minus
    /// that skip.
    abstract_body: f64,
    /// `\end{abstract}` then `\begin{thm}` — the one boundary in this file
    /// whose value is not the same *shape* at every size, and the reason
    /// the sweep is worth its runtime. Both sides are unusual: the
    /// `abstract`'s `\@topsepadd` was fixed inside `\small`, so it is
    /// `\small`'s `\topsep` + `\partopsep` (6/9/12 pt), while an amsthm
    /// theorem opens with `\thm@preskip` = `\normalsize`'s `\topsep`
    /// (8/9/10 pt) and no `\partopsep` at all. `\addvspace` keeps the
    /// larger, and which one that is *changes with the class option*: the
    /// theorem wins at 10 pt, they tie at 11 pt, the `abstract` wins at
    /// 12 pt. A 10 pt-only sweep would have pinned the wrong rule.
    abstract_to_thm: f64,
}

const SIZES: [Size; 3] = [
    Size { opt: "10pt", first: 134.765, adjacent: 21.9175, topsep_only: 19.925, into_abstract: 36.538, abstract_body: 15.616, abstract_to_thm: 19.925 },
    Size { opt: "11pt", first: 140.742, adjacent: 25.5045, topsep_only: 22.516, into_abstract: 42.092, abstract_body: 18.182, abstract_to_thm: 22.516 },
    Size { opt: "12pt", first: 137.753, adjacent: 27.398, topsep_only: 24.409, into_abstract: 46.729, abstract_body: 20.228, abstract_to_thm: 26.401 },
];

/// Every environment measured here, as a single-`\item` block carrying `word`.
fn blk(kind: &str, word: &str) -> String {
    match kind {
        "center" => format!("\\begin{{center}}{word}\\end{{center}}\n"),
        "flushleft" => format!("\\begin{{flushleft}}{word}\\end{{flushleft}}\n"),
        "quote" => format!("\\begin{{quote}}{word}\\end{{quote}}\n"),
        "quotation" => format!("\\begin{{quotation}}{word}\\end{{quotation}}\n"),
        "verse" => format!("\\begin{{verse}}{word}\\end{{verse}}\n"),
        "thm" => format!("\\begin{{thm}}{word}\\end{{thm}}\n"),
        "itemize" => format!("\\begin{{itemize}}\\item {word}\\end{{itemize}}\n"),
        "enumerate" => format!("\\begin{{enumerate}}\\item {word}\\end{{enumerate}}\n"),
        "description" => format!("\\begin{{description}}\\item[Term] {word}\\end{{description}}\n"),
        // `\@verbatim` is `\trivlist \item\relax ...` and `\endverbatim` is
        // `\endtrivlist` (latex.ltx), so `verbatim` is one of these too.
        "verbatim" => format!("\\begin{{verbatim}}\n{word}\n\\end{{verbatim}}\n"),
        // article.cls's one-column `abstract` is `\small`, a centred
        // `\abstractname` and a `quotation`, so `\end{abstract}` is
        // `\endquotation` -> `\endlist` -> `\endtrivlist`.
        "abstract" => format!("\\begin{{abstract}}\n{word}\n\\end{{abstract}}\n"),
        other => panic!("unknown probe environment `{other}`"),
    }
}

const PROBES: [&str; 6] = ["alpha", "bravo", "charlie", "delta", "echo", "foxtrot"];

const LISTS: [&str; 3] = ["itemize", "enumerate", "description"];
/// The `\trivlist`/`\list` environments whose `\item` the compiler does not
/// report as a list item.
const SHAPES: [&str; 7] = ["center", "flushleft", "quote", "quotation", "verse", "thm", "verbatim"];

fn words(size: &str, body: &str) -> Vec<common::Word> {
    common::words_of(&common::render_one(&format!("{}{body}\\end{{document}}\n", head(size))))
}

fn at(words: &[common::Word], text: &str) -> f64 {
    words
        .iter()
        .find(|w| w.text.trim().contains(text))
        .unwrap_or_else(|| panic!("no run `{text}` in {:?}", words.iter().map(|w| &w.text).collect::<Vec<_>>()))
        .baseline
}

fn check(label: &str, got: f64, expect: f64) {
    assert!(
        (got - expect).abs() <= WORD_TOL_BP,
        "{label}: {got:.3} bp, pdflatex {expect:.3} bp ({:+.3})",
        got - expect
    );
}

/// The baselines of `kinds` set one after another, separated by `sep`.
fn chain(kinds: &[&str], sep: &str, size: &str) -> Vec<f64> {
    let body = kinds.iter().enumerate().map(|(i, k)| blk(k, PROBES[i])).collect::<Vec<_>>().join(sep);
    let w = words(size, &body);
    kinds.iter().enumerate().map(|(i, _)| at(&w, PROBES[i])).collect()
}

/// A list that opens right after `center`/`quote`/`quotation`/`verse`/a
/// theorem takes one `\topsep` + `\partopsep`, whether or not a blank line
/// separates them. The theorem is the case that was 1.992 bp tight: its
/// `\end` is an `\endtrivlist` like the others, so the `\begin` after it is
/// read in vertical mode.
#[test]
fn a_list_after_a_paragraph_shape_environment_steps_by_topsep_plus_partopsep() {
    if !common::lm_available() {
        eprintln!("SKIP vmode_boundary_skips: Latin Modern not installed");
        return;
    }
    for s in &SIZES {
        for shape in SHAPES {
            for list in LISTS {
                for (sep, what) in [("", "adjacent"), ("\n", "blank line")] {
                    let b = chain(&[shape, list], sep, s.opt);
                    check(&format!("{} {shape} -> {list} ({what}): alpha", s.opt), b[0], s.first);
                    check(&format!("{} {shape} -> {list} ({what}): bravo", s.opt), b[1], s.first + s.adjacent);
                }
            }
        }
    }
}

/// The other direction, which is where the two skips were *summed*: a
/// `center`/`quote`/`quotation`/`verse`/theorem that opens right after
/// `\end{<list>}` was 18 pt below it instead of 10 pt (+7.971 bp).
#[test]
fn a_paragraph_shape_environment_after_a_list_shares_one_addvspace() {
    if !common::lm_available() {
        eprintln!("SKIP vmode_boundary_skips: Latin Modern not installed");
        return;
    }
    for s in &SIZES {
        for list in LISTS {
            for shape in SHAPES {
                let b = chain(&[list, shape], "", s.opt);
                check(&format!("{} {list} -> {shape}: alpha", s.opt), b[0], s.first);
                check(&format!("{} {list} -> {shape}: bravo", s.opt), b[1], s.first + s.adjacent);
            }
        }
    }
}

/// Two paragraph-shape environments beside each other, including two of the
/// same kind — which the pipeline reads as one `ParaStyle` run, and which
/// therefore lost the boundary's `\partopsep` entirely.
#[test]
fn two_adjacent_paragraph_shape_environments_step_by_topsep_plus_partopsep() {
    if !common::lm_available() {
        eprintln!("SKIP vmode_boundary_skips: Latin Modern not installed");
        return;
    }
    for s in &SIZES {
        for a in SHAPES {
            for b in SHAPES {
                // Two amsthm theorems are the documented exception below.
                if a == "thm" && b == "thm" {
                    continue;
                }
                let bl = chain(&[a, b], "", s.opt);
                check(&format!("{} {a} -> {b}: alpha", s.opt), bl[0], s.first);
                check(&format!("{} {a} -> {b}: bravo", s.opt), bl[1], s.first + s.adjacent);
            }
        }
    }
}

/// The measurement that says whether this is one mishandled skip or a rule:
/// six boundaries in a row, which is where the old error grew to -9.963 bp
/// (all `center`) and +7.970 bp (alternating `center`/`itemize`).
#[test]
fn the_boundary_error_does_not_accumulate_across_six_environments() {
    if !common::lm_available() {
        eprintln!("SKIP vmode_boundary_skips: Latin Modern not installed");
        return;
    }
    let chains: [&[&str]; 8] = [
        &["center"; 6],
        &["quote"; 6],
        &["verbatim"; 6],
        &["center", "itemize", "center", "itemize", "center", "itemize"],
        &["quote", "itemize", "quote", "itemize", "quote", "itemize"],
        &["verbatim", "itemize", "verbatim", "itemize", "verbatim", "itemize"],
        &["center", "verbatim", "quote", "verbatim", "verse", "verbatim"],
        &["center", "quote", "itemize", "verse", "thm", "quotation"],
    ];
    for s in &SIZES {
        for kinds in chains {
            let b = chain(kinds, "", s.opt);
            for (k, probe) in PROBES.iter().enumerate() {
                check(&format!("{} {kinds:?}: {probe}", s.opt), b[k], s.first + s.adjacent * k as f64);
            }
        }
    }
}

/// amsthm assigns `\@topsep`/`\@topsepadd` outright, so a theorem takes no
/// `\partopsep` however it is entered: six theorems in a row step by
/// `\topsep` alone, and a theorem after a blank line is no lower than one
/// after a paragraph. This is the case a `\partopsep` fix must not sweep up.
#[test]
fn an_amsthm_theorem_still_steps_by_topsep_alone() {
    if !common::lm_available() {
        eprintln!("SKIP vmode_boundary_skips: Latin Modern not installed");
        return;
    }
    for s in &SIZES {
        let b = chain(&["thm"; 6], "", s.opt);
        for (k, probe) in PROBES.iter().enumerate() {
            check(&format!("{} six theorems: {probe}", s.opt), b[k], s.first + s.topsep_only * k as f64);
        }
        let w = words(s.opt, &format!("Preceding text paragraph.\n\n{}", blk("thm", "alpha")));
        check(&format!("{} theorem after a blank line", s.opt), at(&w, "alpha") - at(&w, "Preceding"), s.topsep_only);
    }
}

/// The control that separates `\topsep` from `\partopsep` for everything
/// else: a `\begin` read in *horizontal* mode (no blank line after the
/// paragraph) is 1.992 bp higher than one read in vertical mode. Unchanged
/// by the boundary fix, which only adds vertical mode after an
/// `\endtrivlist`.
#[test]
fn partopsep_still_applies_only_in_vertical_mode() {
    if !common::lm_available() {
        eprintln!("SKIP vmode_boundary_skips: Latin Modern not installed");
        return;
    }
    for s in &SIZES {
        for kind in ["center", "quote", "quotation", "verse", "verbatim", "itemize", "enumerate", "description"] {
            let body = blk(kind, "alpha");
            let w = words(s.opt, &format!("Preceding text paragraph.\n{body}"));
            check(&format!("{} {kind} after a paragraph", s.opt), at(&w, "alpha") - at(&w, "Preceding"), s.topsep_only);
            let w = words(s.opt, &format!("Preceding text paragraph.\n\n{body}"));
            check(&format!("{} {kind} after a blank line", s.opt), at(&w, "alpha") - at(&w, "Preceding"), s.adjacent);
            check(&format!("{} {kind} alone", s.opt), at(&words(s.opt, &body), "alpha"), s.first);
        }
    }
}

/// `abstract` is the last `\trivlist`-derived environment #722 left
/// unmeasured, and the one whose two sides differ. Its `\end` is an
/// `\endtrivlist` like every other, so what follows takes `\partopsep`
/// (`\end{abstract}\begin{center}` was 1.992 bp tight at 10 pt, 2.989 at
/// 11 pt, 0.996 at 12 pt). Its `\begin` is a whole shape rather than one
/// skip — a `\small` centred `\abstractname` then a `quotation` — and the
/// pipeline inserts that head as a block of its own in front of the body
/// the compiler produced. The skip the compiler hung on that body (a
/// closing `\end{itemize}`'s `\addvspace\@topsepadd`) has to travel to the
/// head with the position: left behind it fired a second time below the
/// head, and `\end{itemize}\begin{abstract}` came out 5.305 bp long at
/// 10 pt, 5.729 at 11 pt and 6.273 at 12 pt. `\end{center}\begin{abstract}`
/// was already right, because a `center`'s closing skip rides on its own
/// `env_close` rather than on the next block.
#[test]
fn an_abstract_shares_one_addvspace_with_the_environment_on_either_side() {
    if !common::lm_available() {
        eprintln!("SKIP vmode_boundary_skips: Latin Modern not installed");
        return;
    }
    for s in &SIZES {
        for (sep, what) in [("", "adjacent"), ("\n", "blank line")] {
            // Into the abstract, from every `\trivlist` and every list.
            for before in SHAPES.iter().chain(LISTS.iter()) {
                let b = chain(&[before, "abstract"], sep, s.opt);
                check(&format!("{} {before} -> abstract ({what}): alpha", s.opt), b[0], s.first);
                check(&format!("{} {before} -> abstract ({what}): bravo", s.opt), b[1], s.first + s.into_abstract);
            }
            // Out of the abstract: an ordinary `\endtrivlist` boundary.
            for after in SHAPES.iter().chain(LISTS.iter()) {
                let b = chain(&["abstract", after], sep, s.opt);
                check(&format!("{} abstract -> {after} ({what}): alpha", s.opt), b[0], s.first + s.abstract_body);
                check(
                    &format!("{} abstract -> {after} ({what}): bravo", s.opt),
                    b[1],
                    s.first + s.abstract_body + if after == &"thm" { s.abstract_to_thm } else { s.adjacent },
                );
            }
        }
        // The `\abstractname` head sits where a first block does, and the
        // body one `into_abstract` below it.
        let w = words(s.opt, &blk("abstract", "alpha"));
        check(&format!("{} abstract alone: head", s.opt), at(&w, "Abstract"), s.first);
        check(&format!("{} abstract alone: body", s.opt), at(&w, "alpha"), s.first + s.abstract_body);
    }
}
