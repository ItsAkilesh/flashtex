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
//! ## Oracle
//!
//! pdfTeX 1.40 (MacTeX 2026), 10 pt T1 Latin Modern, `\pagestyle{empty}`,
//! PyMuPDF glyph origins, in bp. Every probe document below was run through
//! pdflatex; the constants are its measured baselines. pdflatex is an oracle
//! only and never runs in the product path.

mod common;

/// Word x/baseline gate for this project.
const WORD_TOL_BP: f64 = 0.5;

const HEAD: &str = "\\documentclass[10pt]{article}\n\\usepackage[T1]{fontenc}\n\\usepackage{lmodern}\n\\usepackage{amsthm}\n\\newtheorem{thm}{Theorem}\n\\pagestyle{empty}\n\\begin{document}\n";

/// pdflatex's first baseline on an empty `article` page.
const FIRST: f64 = 134.765;
/// The step from one `\trivlist` environment to the next one beside it:
/// `\baselineskip` 12 pt + max(`\topsep` 8 pt + `\partopsep` 2 pt, the same).
const ADJACENT: f64 = 21.9175;
/// The step when only `\topsep` applies (`\baselineskip` 12 pt + 8 pt): a
/// `\begin` read in horizontal mode, or an amsthm theorem, which sets
/// `\@topsep`/`\@topsepadd` itself and so never takes `\partopsep`.
const TOPSEP_ONLY: f64 = 19.925;

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
        other => panic!("unknown probe environment `{other}`"),
    }
}

const PROBES: [&str; 6] = ["alpha", "bravo", "charlie", "delta", "echo", "foxtrot"];

const LISTS: [&str; 3] = ["itemize", "enumerate", "description"];
/// The `\trivlist`/`\list` environments whose `\item` the compiler does not
/// report as a list item.
const SHAPES: [&str; 6] = ["center", "flushleft", "quote", "quotation", "verse", "thm"];

fn words(body: &str) -> Vec<common::Word> {
    common::words_of(&common::render_one(&format!("{HEAD}{body}\\end{{document}}\n")))
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
fn chain(kinds: &[&str], sep: &str) -> Vec<f64> {
    let body = kinds.iter().enumerate().map(|(i, k)| blk(k, PROBES[i])).collect::<Vec<_>>().join(sep);
    let w = words(&body);
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
    for shape in SHAPES {
        for list in LISTS {
            for (sep, what) in [("", "adjacent"), ("\n", "blank line")] {
                let b = chain(&[shape, list], sep);
                check(&format!("{shape} -> {list} ({what}): alpha"), b[0], FIRST);
                check(&format!("{shape} -> {list} ({what}): bravo"), b[1], FIRST + ADJACENT);
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
    for list in LISTS {
        for shape in SHAPES {
            let b = chain(&[list, shape], "");
            check(&format!("{list} -> {shape}: alpha"), b[0], FIRST);
            check(&format!("{list} -> {shape}: bravo"), b[1], FIRST + ADJACENT);
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
    for a in SHAPES {
        for b in SHAPES {
            // Two amsthm theorems are the documented exception below.
            if a == "thm" && b == "thm" {
                continue;
            }
            let bl = chain(&[a, b], "");
            check(&format!("{a} -> {b}: alpha"), bl[0], FIRST);
            check(&format!("{a} -> {b}: bravo"), bl[1], FIRST + ADJACENT);
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
    let chains: [&[&str]; 5] = [
        &["center"; 6],
        &["quote"; 6],
        &["center", "itemize", "center", "itemize", "center", "itemize"],
        &["quote", "itemize", "quote", "itemize", "quote", "itemize"],
        &["center", "quote", "itemize", "verse", "thm", "quotation"],
    ];
    for kinds in chains {
        let b = chain(kinds, "");
        for (k, probe) in PROBES.iter().enumerate() {
            check(&format!("{kinds:?}: {probe}"), b[k], FIRST + ADJACENT * k as f64);
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
    let b = chain(&["thm"; 6], "");
    for (k, probe) in PROBES.iter().enumerate() {
        check(&format!("six theorems: {probe}"), b[k], FIRST + TOPSEP_ONLY * k as f64);
    }
    let w = words(&format!("Preceding text paragraph.\n\n{}", blk("thm", "alpha")));
    check("theorem after a blank line", at(&w, "alpha") - at(&w, "Preceding"), TOPSEP_ONLY);
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
    for kind in ["center", "quote", "quotation", "verse", "itemize", "enumerate", "description"] {
        let body = blk(kind, "alpha");
        let w = words(&format!("Preceding text paragraph.\n{body}"));
        check(&format!("{kind} after a paragraph"), at(&w, "alpha") - at(&w, "Preceding"), TOPSEP_ONLY);
        let w = words(&format!("Preceding text paragraph.\n\n{body}"));
        check(&format!("{kind} after a blank line"), at(&w, "alpha") - at(&w, "Preceding"), ADJACENT);
        check(&format!("{kind} alone"), at(&words(&body), "alpha"), FIRST);
    }
}
