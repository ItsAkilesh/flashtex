//! FT-041 rev 4, objective 2: measured, not listed, LaTeX-snippet coverage.
//!
//! Every `template` below is a realistic snippet an editor's LaTeX snippet
//! library would plausibly offer for a construct that actually appears in
//! `tests/tex-corpus/cases/**/main.tex` (transcribed verbatim in the
//! `// corpus:` comment beside each candidate — this crate does not read
//! that directory at test time, since it belongs to a different owner's
//! fixtures, not a dependency of this crate). Each is tagged with the
//! fuller-LSP feature it would need *if* this crate's grammar already
//! handled plain LaTeX text, and this test asserts the *exact* typed
//! outcome `Snippet::parse` returns today, then pins the aggregate counts
//! so the measurement cannot silently drift.
//!
//! Corpus commands/environments actually present (read directly from the
//! `.tex` files under `tests/tex-corpus/cases/`, 2026-09-12):
//! `\documentclass`, `\begin`/`\end` (`document`, `tikzpicture`),
//! `\newcommand`, `\renewcommand`, `\input`, `\usepackage`, `\draw`,
//! `\frac`, inline `$...$` and display `\[...\]` math, and the escaped
//! LaTeX specials `\$ \% \& \_ \#`.

use flashtex_editor_snippets::{Snippet, SnippetError};

/// Which fuller-LSP-grammar feature a candidate's *intended* semantics
/// would need, independent of whether it also happens to trip the
/// backslash-escape finding below.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Feature {
    /// Numbered placeholders, defaults, linked occurrences, `$0` — rev 1's
    /// grammar. No fuller-LSP feature should be needed.
    None,
    /// `${1|choice,choice|}`.
    Choice,
    /// A non-numeric/special variable, e.g. `$TM_SELECTED_TEXT`.
    NamedVariable,
    /// `${1/regex/replacement/flags}`.
    Transform,
}

/// The exact result `Snippet::parse` must return for a candidate.
enum Check {
    /// Parses, and expands to exactly this text (no overrides).
    ParsesAndExpandsTo(&'static str),
    /// Rejected with this exact typed error.
    Rejected(SnippetError),
}

struct Candidate {
    label: &'static str,
    feature: Feature,
    template: &'static str,
    check: Check,
}

fn is_invalid_escape(err: &SnippetError) -> bool {
    matches!(err, SnippetError::InvalidEscape { .. })
}

fn run(candidates: &[Candidate]) {
    for c in candidates {
        match (&c.check, Snippet::parse(c.template)) {
            (Check::ParsesAndExpandsTo(expected), Ok(snippet)) => {
                let expansion = snippet
                    .expand()
                    .unwrap_or_else(|e| panic!("{}: expected to expand, got {e:?}", c.label));
                assert_eq!(
                    expansion.text, *expected,
                    "{}: unexpected expansion",
                    c.label
                );
            }
            (Check::ParsesAndExpandsTo(_), Err(e)) => {
                panic!("{}: expected to parse, got {e:?}", c.label);
            }
            (Check::Rejected(expected), Err(actual)) => {
                assert_eq!(&actual, expected, "{}: wrong rejection reason", c.label);
            }
            (Check::Rejected(_), Ok(_)) => {
                panic!("{}: expected rejection, but it parsed", c.label);
            }
        }
    }
}

/// 22 realistic snippet shapes, each grounded in a real corpus construct,
/// written the way a real LaTeX snippet library (and the corpus itself)
/// naturally writes LaTeX: a single backslash before a command name.
fn corpus_derived_candidates() -> Vec<Candidate> {
    vec![
        // corpus: `\documentclass{article}` (every case file)
        Candidate {
            label: "documentclass_default",
            feature: Feature::None,
            template: "\\documentclass{${1:article}}",
            check: Check::Rejected(SnippetError::InvalidEscape {
                offset: 0,
                found: 'd',
            }),
        },
        Candidate {
            label: "documentclass_choice",
            feature: Feature::Choice,
            template: "\\documentclass[${1:11pt}]{${2|article,report,book,letter|}}",
            check: Check::Rejected(SnippetError::InvalidEscape {
                offset: 0,
                found: 'd',
            }),
        },
        // corpus: `\begin{document}` ... `\end{document}` (every case file)
        Candidate {
            label: "begin_end_numbered",
            feature: Feature::None,
            template: "\\begin{${1:document}}\n$0\n\\end{$1}",
            check: Check::Rejected(SnippetError::InvalidEscape {
                offset: 0,
                found: 'b',
            }),
        },
        Candidate {
            label: "begin_end_choice",
            feature: Feature::Choice,
            template: "\\begin{${1|itemize,enumerate,description|}}\n\t$0\n\\end{$1}",
            check: Check::Rejected(SnippetError::InvalidEscape {
                offset: 0,
                found: 'b',
            }),
        },
        // corpus: `\newcommand{\word}{outer}` (macro-arguments, macro-scope, include-scope)
        Candidate {
            label: "newcommand_numbered",
            feature: Feature::None,
            template: "\\newcommand{\\${1:name}}{${2:definition}}",
            check: Check::Rejected(SnippetError::InvalidEscape {
                offset: 0,
                found: 'n',
            }),
        },
        // corpus: `\renewcommand{\word}{inner}` (include-scope/local.tex, macro-scope)
        Candidate {
            label: "renewcommand_numbered",
            feature: Feature::None,
            template: "\\renewcommand{\\${1:name}}{${2:definition}}",
            check: Check::Rejected(SnippetError::InvalidEscape {
                offset: 0,
                found: 'r',
            }),
        },
        Candidate {
            label: "renewcommand_named_selection",
            feature: Feature::NamedVariable,
            template: "\\renewcommand{\\${1:name}}{${TM_SELECTED_TEXT}}",
            check: Check::Rejected(SnippetError::InvalidEscape {
                offset: 0,
                found: 'r',
            }),
        },
        // corpus: `\newcommand{\pair}[2]{#1 then #2}` (macro-arguments)
        Candidate {
            label: "newcommand_transform_args",
            feature: Feature::Transform,
            template: "\\newcommand{\\${1:name}}[${2:2}]{${2/(\\d+)/#$1 /g}}",
            check: Check::Rejected(SnippetError::InvalidEscape {
                offset: 0,
                found: 'n',
            }),
        },
        // corpus: `\input{parts/section}` / `\input{local}` (included-file, include-scope, missing-include)
        Candidate {
            label: "input_numbered",
            feature: Feature::None,
            template: "\\input{${1:filename}}",
            check: Check::Rejected(SnippetError::InvalidEscape {
                offset: 0,
                found: 'i',
            }),
        },
        Candidate {
            label: "input_named_basename",
            feature: Feature::NamedVariable,
            template: "\\input{${TM_FILENAME_BASE}_${1:part}}",
            check: Check::Rejected(SnippetError::InvalidEscape {
                offset: 0,
                found: 'i',
            }),
        },
        // corpus: `\usepackage{tikz}` (tikz-required)
        Candidate {
            label: "usepackage_numbered",
            feature: Feature::None,
            template: "\\usepackage{${1:tikz}}",
            check: Check::Rejected(SnippetError::InvalidEscape {
                offset: 0,
                found: 'u',
            }),
        },
        Candidate {
            label: "usepackage_choice",
            feature: Feature::Choice,
            template: "\\usepackage{${1|tikz,amsmath,graphicx,hyperref|}}",
            check: Check::Rejected(SnippetError::InvalidEscape {
                offset: 0,
                found: 'u',
            }),
        },
        // corpus: `\begin{tikzpicture}` + `\draw (0,0) -- (1,0) -- (1,1);` (tikz-required)
        Candidate {
            label: "tikzpicture_draw_numbered",
            feature: Feature::None,
            template: "\\begin{tikzpicture}\n\t\\draw (${1:0,0}) -- (${2:1,0});\n\\end{tikzpicture}",
            check: Check::Rejected(SnippetError::InvalidEscape {
                offset: 0,
                found: 'b',
            }),
        },
        Candidate {
            label: "draw_choice_style",
            feature: Feature::Choice,
            template: "\\draw[${1|thick,dashed,dotted|}] (${2:0,0}) -- (${3:1,1});",
            check: Check::Rejected(SnippetError::InvalidEscape {
                offset: 0,
                found: 'd',
            }),
        },
        // corpus: `\frac{a+b}{c}` (math-inline-display)
        Candidate {
            label: "frac_numbered",
            feature: Feature::None,
            template: "\\frac{${1:a}}{${2:b}}",
            check: Check::Rejected(SnippetError::InvalidEscape {
                offset: 0,
                found: 'f',
            }),
        },
        // corpus: `\[\frac{a+b}{c}=d\]` (math-inline-display)
        Candidate {
            label: "display_math_numbered",
            feature: Feature::None,
            template: "\\[${1:x}\\]",
            check: Check::Rejected(SnippetError::InvalidEscape {
                offset: 0,
                found: '[',
            }),
        },
        Candidate {
            label: "display_math_named_selection",
            feature: Feature::NamedVariable,
            template: "\\[\n\t${1:${TM_SELECTED_TEXT}}\n\\]",
            check: Check::Rejected(SnippetError::InvalidEscape {
                offset: 0,
                found: '[',
            }),
        },
        // corpus: `Inline $x_1^2 + y$ ends.` (math-inline-display) — this
        // one only needs `\$`, the one escape this crate already supports.
        Candidate {
            label: "inline_math_via_dollar_escape",
            feature: Feature::None,
            template: "\\$${1:x}\\$",
            check: Check::ParsesAndExpandsTo("$x$"),
        },
        // corpus: `Price \$5; 50\%; A\&B; C\_D; \#1.` (comments-escapes)
        Candidate {
            label: "percent_literal_numbered",
            feature: Feature::None,
            template: "${1:50}\\%",
            check: Check::Rejected(SnippetError::InvalidEscape {
                offset: 7,
                found: '%',
            }),
        },
        Candidate {
            label: "ampersand_literal_numbered",
            feature: Feature::None,
            template: "${1:A}\\&${2:B}",
            check: Check::Rejected(SnippetError::InvalidEscape {
                offset: 6,
                found: '&',
            }),
        },
        Candidate {
            label: "underscore_literal_numbered",
            feature: Feature::None,
            template: "${1:C}\\_${2:D}",
            check: Check::Rejected(SnippetError::InvalidEscape {
                offset: 6,
                found: '_',
            }),
        },
        Candidate {
            label: "hash_literal_numbered",
            feature: Feature::None,
            template: "\\#${1:1}",
            check: Check::Rejected(SnippetError::InvalidEscape {
                offset: 0,
                found: '#',
            }),
        },
    ]
}

/// Isolated probes with *no* LaTeX command backslash at all, so a fuller-LSP
/// feature's own support (or lack of it) is not confounded with the
/// backslash-escape finding measured above.
fn isolated_feature_probes() -> Vec<Candidate> {
    vec![
        Candidate {
            label: "isolated_choice",
            feature: Feature::Choice,
            template: "${1|itemize,enumerate,description|}",
            check: Check::Rejected(SnippetError::UnterminatedPlaceholder { offset: 0 }),
        },
        Candidate {
            label: "isolated_named_variable_braced",
            feature: Feature::NamedVariable,
            template: "${TM_SELECTED_TEXT}",
            check: Check::Rejected(SnippetError::InvalidPlaceholderIndex { offset: 2 }),
        },
        Candidate {
            label: "isolated_transform",
            feature: Feature::Transform,
            template: "${1/(.*)/\\U$1/}",
            check: Check::Rejected(SnippetError::UnterminatedPlaceholder { offset: 0 }),
        },
    ]
}

/// A fourth isolated probe, kept separate from the others: a bare `$NAME`
/// (no braces) does not error at all. `parse_dollar` treats any `$` not
/// followed by a digit or `{` as literal text (src/parser.rs), so the whole
/// would-be variable reference is silently copied through as dead text —
/// parsing "succeeds" while dropping the intended substitution entirely,
/// which is a worse failure mode than a typed rejection: nothing signals
/// that anything was lost.
#[test]
fn bare_named_variable_silently_misparses_instead_of_erroring() {
    let template = "wrap: $TM_SELECTED_TEXT end";
    let snippet = Snippet::parse(template).expect("a bare $NAME does not error");
    let expansion = snippet.expand().unwrap();
    assert_eq!(
        expansion.text, template,
        "no placeholder was recognized; the text is copied through unchanged"
    );
}

#[test]
fn corpus_derived_snippet_shapes_match_their_exact_measured_outcome() {
    let candidates = corpus_derived_candidates();
    run(&candidates);

    let total = candidates.len();
    let parses = candidates
        .iter()
        .filter(|c| matches!(c.check, Check::ParsesAndExpandsTo(_)))
        .count();
    let backslash_blocked = candidates
        .iter()
        .filter(|c| matches!(&c.check, Check::Rejected(e) if is_invalid_escape(e)))
        .count();
    let by_feature = |feature: Feature| candidates.iter().filter(|c| c.feature == feature).count();

    // Pinned so this measurement cannot silently drift: 22 realistic
    // corpus-derived shapes, only 1 parses today, and every failure is the
    // same InvalidEscape cause -- this crate's `\` is reserved for its own
    // escape grammar (`\$ \} \\` only, src/parser.rs::parse_escape), so a
    // literal LaTeX command name or LaTeX's own escaped specials
    // (`\% \& \_ \#`) are rejected before the placeholder grammar (numbered,
    // choice, named, transform) is ever reached. This dominates all three
    // previously-documented gaps combined: 13 of the 14 `Feature::None`
    // shapes -- which need *no* fuller-LSP feature at all -- fail on this
    // alone.
    assert_eq!(total, 22);
    assert_eq!(parses, 1);
    assert_eq!(backslash_blocked, total - parses);
    assert_eq!(by_feature(Feature::None), 14);
    assert_eq!(by_feature(Feature::Choice), 4);
    assert_eq!(by_feature(Feature::NamedVariable), 3);
    assert_eq!(by_feature(Feature::Transform), 1);
}

#[test]
fn isolated_lsp_feature_probes_match_their_exact_measured_outcome() {
    // Free of the backslash confound above, each of the three previously-
    // documented gaps is independently confirmed unsupported: choice and
    // transform are hard rejections; a braced named variable is too, and a
    // bare one is the silent misparse covered by its own test above.
    run(&isolated_feature_probes());
}
