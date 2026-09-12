//! Behavioural tests for `explain`: offsets on multi-byte text, environment
//! insertion points, closest-command lookup, math, context bounds, fallback.

use flashtex_diagnostic_explanations::{
    Category, Confidence, DEFAULT_SUPPORTED_COMMANDS, Diagnostic, Document, Edit, Severity,
    explain, explain_all, preview,
};

const P: &str = "main.tex";

fn err(message: &str, doc: &str, needle: &str, recovery: &str) -> Diagnostic {
    let s = doc.find(needle).expect("needle present");
    Diagnostic::at(Severity::Error, message, P, s, s + needle.len(), recovery)
}

fn apply(doc: &str, edits: &[Edit]) -> String {
    preview::apply_edits(doc, edits).unwrap()
}

#[test]
fn unclosed_group_offsets_are_bytes_on_multibyte_text() {
    // Mirrors tests/tex-corpus unclosed-group: the '{' is reported.
    let doc = "Über {inside ünd more\n\nNächster Absatz.";
    let d = err(
        "unmatched '{' — group never closed",
        doc,
        "{",
        "treated the rest of the document as part of the group",
    );
    let x = explain(&d, doc, DEFAULT_SUPPORTED_COMMANDS);
    assert_eq!(x.category, Category::UnmatchedBrace);
    assert_eq!(x.catalog_id.as_deref(), Some("unclosed-group"));
    let first = &x.suggestions[0];
    assert_eq!(first.confidence, Confidence::Medium);
    // Insert '}' after "more", which sits after two multi-byte characters.
    let insert_at = doc.find("more").unwrap() + 4;
    assert_eq!(first.edits, vec![Edit::insert(P, insert_at, "}")]);
    assert_eq!(
        apply(doc, &first.edits),
        "Über {inside ünd more}\n\nNächster Absatz."
    );
    assert!(x.what_happened.starts_with("The compiler treated the rest"));
    let ctx = x.context.as_ref().unwrap();
    assert_eq!((ctx.line, ctx.column), (1, 6));
}

#[test]
fn stray_close_brace_is_removed_or_escaped() {
    // Mirrors tests/tex-corpus extra-closing-group.
    let doc = "Text with an extra } brace.";
    let d = err(
        "unmatched '}' — no group is open here",
        doc,
        "}",
        "ignored the stray brace and continued",
    );
    let x = explain(&d, doc, &[]);
    assert_eq!(
        apply(doc, &x.suggestions[0].edits),
        "Text with an extra  brace."
    );
    assert_eq!(
        apply(doc, &x.suggestions[1].edits),
        "Text with an extra \\} brace."
    );
}

#[test]
fn argument_unclosed_closes_before_paragraph_break() {
    let doc = "\\textbf{bold ünd\nmore\n\nNext.";
    let d = err(
        "argument to \\textbf is missing its closing brace",
        doc,
        "{",
        "closed the argument at end of input",
    );
    let x = explain(&d, doc, &[]);
    assert_eq!(
        apply(doc, &x.suggestions[0].edits),
        "\\textbf{bold ünd\nmore}\n\nNext."
    );
}

#[test]
fn unterminated_environment_inserts_end_before_next_section() {
    let doc = "\\section{Én}\n\\begin{itemize}\nitem ✓\n\\begin{center}x\\end{center}\n\\section{Two}\nend\n";
    let begin = doc.find("\\begin{itemize}").unwrap();
    let d = Diagnostic::at(
        Severity::Error,
        "unterminated environment 'itemize' — no matching \\end",
        P,
        begin,
        begin + "\\begin{itemize}".len(),
        "closed the environment at end of input",
    );
    let x = explain(&d, doc, &[]);
    assert_eq!(x.category, Category::EnvironmentMismatch);
    let s = &x.suggestions[0];
    assert_eq!(s.confidence, Confidence::Medium);
    let expected_at = doc.find("\\section{Two}").unwrap();
    assert_eq!(
        s.edits,
        vec![Edit::insert(P, expected_at, "\\end{itemize}\n")]
    );
    assert!(apply(doc, &s.edits).contains("\\end{center}\n\\end{itemize}\n\\section{Two}"));
    // Every candidate is a distinct position and none precede the \begin.
    let mut seen = Vec::new();
    for s in &x.suggestions {
        let at = s.edits[0].start_byte;
        assert!(at >= begin + "\\begin{itemize}".len());
        assert!(!seen.contains(&at), "duplicate insertion point");
        seen.push(at);
    }
}

#[test]
fn unterminated_document_closes_at_end_of_file() {
    let doc = "\\begin{document}\nHello\n\n";
    let d = err(
        "unterminated environment 'document' — no matching \\end",
        doc,
        "\\begin{document}",
        "closed the environment at end of input",
    );
    let x = explain(&d, doc, &[]);
    assert_eq!(x.suggestions.len(), 1);
    assert_eq!(x.suggestions[0].confidence, Confidence::High);
    assert_eq!(
        apply(doc, &x.suggestions[0].edits),
        "\\begin{document}\nHello\n\\end{document}\n"
    );
}

#[test]
fn environment_mismatch_renames_the_end() {
    let doc = "\\begin{itemize}\nx\n\\end{itemise}";
    let d = err(
        "\\end{itemise} does not match \\begin{itemize}",
        doc,
        "\\end",
        "closed the innermost open environment",
    );
    let x = explain(&d, doc, &[]);
    assert_eq!(x.title, "\\end{itemise} closes \\begin{itemize}");
    assert_eq!(x.suggestions[0].confidence, Confidence::High);
    assert_eq!(
        apply(doc, &x.suggestions[0].edits),
        "\\begin{itemize}\nx\n\\end{itemize}"
    );
    assert_eq!(
        apply(doc, &x.suggestions[1].edits),
        "\\begin{itemize}\nx\n\\end{itemize}\n\\end{itemise}"
    );
}

#[test]
fn stray_end_is_removed() {
    let doc = "a\n\\end{itemize} b";
    let d = err(
        "\\end{itemize} with no matching \\begin",
        doc,
        "\\end",
        "ignored the stray \\end",
    );
    let x = explain(&d, doc, &[]);
    assert_eq!(apply(doc, &x.suggestions[0].edits), "a\n b");
}

#[test]
fn unsupported_command_suggests_closest_supported_name() {
    // Mirrors tests/tex-corpus unknown-command with a near-miss spelling.
    let doc = "Intro\n\\sectoin{Title}\nbody";
    let d = err(
        "\\sectoin is not supported by this compiler version",
        doc,
        "\\sectoin",
        "skipped the command; any braced argument was typeset as plain text",
    );
    let x = explain(&d, doc, DEFAULT_SUPPORTED_COMMANDS);
    assert_eq!(x.category, Category::UnsupportedCommand);
    assert_eq!(x.title, "\\sectoin is not supported");
    let s = &x.suggestions[0];
    assert_eq!(s.text, "Did you mean \\section?");
    assert_eq!(s.confidence, Confidence::High); // one transposition = distance 1 (OSA)
    assert_eq!(apply(doc, &s.edits), "Intro\n\\section{Title}\nbody");
    let keep = x
        .suggestions
        .iter()
        .find(|s| s.text.starts_with("Remove"))
        .unwrap();
    assert_eq!(apply(doc, &keep.edits), "Intro\nTitle\nbody");
    // The app's list is authoritative: without it there is no rename.
    let none = explain(&d, doc, &[]);
    assert!(
        none.suggestions
            .iter()
            .all(|s| !s.text.starts_with("Did you mean"))
    );
    assert!(none.suggestions.iter().any(|s| s.edits.is_empty()));
}

#[test]
fn foundation_branch_unsupported_wording_shares_the_entry() {
    let doc = "\\tikz x";
    let d = err(
        "\\tikz is not supported by this compiler version; unrestricted TeX math mode is not implemented",
        doc,
        "\\tikz",
        "skipped the command; any braced argument was typeset as plain text",
    );
    let x = explain(&d, doc, DEFAULT_SUPPORTED_COMMANDS);
    assert_eq!(x.catalog_id.as_deref(), Some("unsupported-command"));
    assert!(
        x.suggestions
            .iter()
            .all(|s| !s.text.starts_with("Did you mean"))
    );
}

#[test]
fn missing_argument_wraps_the_heading_text() {
    let doc = "\\section Ünïcode heading\nbody";
    let d = err(
        "\\section requires a braced argument",
        doc,
        "\\section",
        "used an empty argument and continued",
    );
    let x = explain(&d, doc, &[]);
    assert_eq!(
        apply(doc, &x.suggestions[0].edits),
        "\\section{Ünïcode heading}\nbody"
    );
    assert_eq!(
        apply(doc, &x.suggestions[1].edits),
        "\\section{} Ünïcode heading\nbody"
    );
}

#[test]
fn math_on_main_offers_removing_the_dollars() {
    let doc = "Let $x^2 + ÿ$ hold.";
    let d = err(
        "math mode is not implemented in this version",
        doc,
        "$",
        "skipped the math shift character; no math was typeset",
    );
    let x = explain(&d, doc, DEFAULT_SUPPORTED_COMMANDS);
    assert_eq!(x.category, Category::MathUnsupported);
    assert_eq!(apply(doc, &x.suggestions[0].edits), "Let x^2 + ÿ hold.");
    assert_eq!(apply(doc, &x.suggestions[1].edits), "Let  hold.");
    assert!(x.suggestions.last().unwrap().edits.is_empty());
}

#[test]
fn unsupported_math_command_uses_the_supported_subset() {
    let doc = "$\\alhpa + \\mathbb{R}$";
    let d = err(
        "\\alhpa is not supported in math mode",
        doc,
        "\\alhpa",
        "typeset the command literally and continued",
    );
    let x = explain(&d, doc, DEFAULT_SUPPORTED_COMMANDS);
    assert_eq!(x.suggestions[0].text, "Did you mean \\alpha?");
    assert_eq!(
        apply(doc, &x.suggestions[0].edits),
        "$\\alpha + \\mathbb{R}$"
    );

    let d = err(
        "\\mathbb is not supported in math mode",
        doc,
        "\\mathbb",
        "typeset the command literally and continued",
    );
    let x = explain(&d, doc, DEFAULT_SUPPORTED_COMMANDS);
    assert!(
        x.suggestions
            .iter()
            .any(|s| s.text.contains("Supported in math"))
    );
    assert_eq!(apply(doc, &x.suggestions[0].edits), "$\\alhpa + {R}$");
}

#[test]
fn script_outside_math_wraps_or_escapes() {
    let doc = "value x_i here";
    let d = err(
        "math script marker used outside math mode",
        doc,
        "_",
        "ignored the script marker and continued",
    );
    let x = explain(&d, doc, &[]);
    assert_eq!(apply(doc, &x.suggestions[0].edits), "value $x_i$ here");
    assert_eq!(apply(doc, &x.suggestions[1].edits), "value x\\_i here");
    assert_eq!(x.suggestions[1].confidence, Confidence::Medium);
}

#[test]
fn duplicate_script_groups_the_first_script() {
    let doc = "$x^{a}^b$";
    let second = doc.rfind('^').unwrap();
    let d = Diagnostic::at(
        Severity::Error,
        "duplicate script on a math atom",
        P,
        second,
        second + 1,
        "used the last script and continued",
    );
    let x = explain(&d, doc, &[]);
    assert_eq!(apply(doc, &x.suggestions[0].edits), "${x^{a}}^b$");
}

#[test]
fn inline_math_unclosed_closes_at_line_end() {
    let doc = "Let $x = ü be\n\nnext";
    let d = err(
        "inline math is missing its closing '$'",
        doc,
        "$",
        "closed math mode at end of input and typeset its contents",
    );
    let x = explain(&d, doc, &[]);
    assert_eq!(
        apply(doc, &x.suggestions[0].edits),
        "Let $x = ü be$\n\nnext"
    );
}

#[test]
fn preamble_command_is_moved_into_the_body() {
    let doc =
        "\\documentclass{article}\n\\section{Early}\n\\begin{document}\nbody\n\\end{document}\n";
    let d = err(
        "\\section is not supported in the document preamble",
        doc,
        "\\section",
        "skipped the command and did not typeset preamble content",
    );
    let x = explain(&d, doc, &[]);
    assert_eq!(x.category, Category::PreambleUnsupported);
    assert_eq!(
        apply(doc, &x.suggestions[0].edits),
        "\\documentclass{article}\n\\begin{document}\n\\section{Early}\nbody\n\\end{document}\n"
    );
}

#[test]
fn macro_definer_is_swapped_with_high_confidence() {
    let doc = "\\newcommand{\\same}{old}\\newcommand{\\same}{new}";
    let second = doc.rfind("\\newcommand").unwrap();
    let d = Diagnostic::at(
        Severity::Error,
        "\\newcommand cannot redefine existing command \\same",
        P,
        second,
        second + "\\newcommand{\\same}".len(),
        "kept the existing command definition",
    );
    let x = explain(&d, doc, &[]);
    assert_eq!(x.suggestions[0].confidence, Confidence::High);
    assert_eq!(
        apply(doc, &x.suggestions[0].edits),
        "\\newcommand{\\same}{old}\\renewcommand{\\same}{new}"
    );
}

#[test]
fn macro_argument_undeclared_edits_the_definition() {
    let doc = "\\newcommand{\\greet}{Hello #1} \\greet{world}";
    let d = err(
        "macro replacement references #1 but that argument is not declared",
        doc,
        "\\greet{world}",
        "omitted the unavailable argument",
    );
    let d = Diagnostic {
        source: d.source.map(|mut s| {
            s.end_byte = s.start_byte + "\\greet".len();
            s
        }),
        ..d
    };
    let x = explain(&d, doc, &[]);
    assert_eq!(
        apply(doc, &x.suggestions[0].edits),
        "\\newcommand{\\greet}[1]{Hello #1} \\greet{world}"
    );
}

#[test]
fn path_rejected_gives_advice_without_edits() {
    let d = Diagnostic::new(
        Severity::Error,
        "rejected document path '../etc/passwd': paths must be project-relative with no parent traversal",
        None,
        None,
    );
    let x = explain(&d, "", &[]);
    assert_eq!(x.category, Category::PathRejected);
    assert!(x.context.is_none());
    assert!(x.suggestions.iter().all(|s| s.edits.is_empty()));
    assert!(x.suggestions[0].text.contains("'etc/passwd'"));
    assert_eq!(
        x.what_happened,
        "The compiler recorded the error and did not recover any output for it."
    );
}

#[test]
fn context_window_is_bounded_and_never_splits_a_scalar() {
    let filler = "😀".repeat(150); // 600 bytes on one line
    let doc = format!("{filler}\\bogus{filler}\n");
    let d = err(
        "\\bogus is not supported by this compiler version",
        &doc,
        "\\bogus",
        "skipped",
    );
    let x = explain(&d, &doc, &[]);
    let ctx = x.context.unwrap();
    assert!(ctx.span_in_bounds);
    assert!(ctx.span_start - ctx.start_byte <= 200);
    assert!(ctx.end_byte - ctx.span_end <= 200);
    assert!(doc.is_char_boundary(ctx.start_byte) && doc.is_char_boundary(ctx.end_byte));
    let (a, b) = ctx.span_in_window();
    assert_eq!(&ctx.text[a..b], "\\bogus");
    assert!(ctx.text.chars().all(|c| c == '😀' || c.is_ascii()));
}

#[test]
fn stale_span_degrades_to_advice() {
    let doc = "short";
    let d = Diagnostic::at(
        Severity::Error,
        "unmatched '}' — no group is open here",
        P,
        40,
        41,
        "ignored the stray brace and continued",
    );
    let x = explain(&d, doc, &[]);
    assert!(!x.context.as_ref().unwrap().span_in_bounds);
    assert!(x.suggestions.iter().all(|s| s.edits.is_empty()));
}

#[test]
fn unknown_message_falls_back_with_context_and_generic_suggestion() {
    let doc = "hello wörld";
    let d = err(
        "something entirely new happened; details",
        doc,
        "wörld",
        "did a thing",
    );
    let x = explain(&d, doc, DEFAULT_SUPPORTED_COMMANDS);
    assert_eq!(x.catalog_id, None);
    assert_eq!(x.category, Category::Unknown);
    assert_eq!(x.title, "Something entirely new happened");
    assert_eq!(x.what_happened, "The compiler did a thing.");
    assert_eq!(x.suggestions.len(), 1);
    assert!(x.suggestions[0].edits.is_empty());
    let ctx = x.context.unwrap();
    assert_eq!(ctx.text, doc);
    assert_eq!((ctx.line, ctx.column), (1, 7));
}

#[test]
fn warning_without_recovery_says_so() {
    let d = Diagnostic::new(
        Severity::Warning,
        "3 documents were supplied; this version compiles only the entry document",
        None,
        None,
    );
    let x = explain(&d, "", &[]);
    assert_eq!(x.category, Category::MultiFile);
    assert_eq!(x.severity, Severity::Warning);
    assert!(x.what_happened.contains("continued"));
}

#[test]
fn batch_api_matches_documents_by_path_and_preserves_order() {
    let main = "a {b";
    let other = "\\bogus";
    let diags = vec![
        Diagnostic::at(
            Severity::Error,
            "unmatched '{' — group never closed",
            "main.tex",
            2,
            3,
            "treated the rest of the document as part of the group",
        ),
        Diagnostic::at(
            Severity::Error,
            "\\bogus is not supported by this compiler version",
            "other.tex",
            0,
            6,
            "skipped",
        ),
        Diagnostic::at(
            Severity::Error,
            "unmatched '}' — no group is open here",
            "missing.tex",
            0,
            1,
            "ignored",
        ),
    ];
    let docs = [
        Document {
            path: "main.tex",
            text: main,
        },
        Document {
            path: "other.tex",
            text: other,
        },
    ];
    let xs = explain_all(&diags, &docs, DEFAULT_SUPPORTED_COMMANDS);
    assert_eq!(xs.len(), 3);
    assert_eq!(xs[0].context.as_ref().unwrap().text, main);
    assert_eq!(xs[1].context.as_ref().unwrap().text, other);
    assert!(!xs[2].context.as_ref().unwrap().span_in_bounds);
    assert_eq!(xs[0].suggestions[0].edits[0].path, "main.tex");
}

#[test]
fn explanations_are_deterministic() {
    let doc = "\\begin{itemize}\n\\itme x\n\\sectoin{y}\n";
    let d = err(
        "\\sectoin is not supported by this compiler version",
        doc,
        "\\sectoin",
        "skipped",
    );
    let a = explain(&d, doc, DEFAULT_SUPPORTED_COMMANDS);
    for _ in 0..5 {
        let b = explain(&d, doc, DEFAULT_SUPPORTED_COMMANDS);
        assert_eq!(a, b);
        assert_eq!(a.to_json(), b.to_json());
    }
    // Supported-list order does not change the ranking.
    let mut reversed: Vec<&str> = DEFAULT_SUPPORTED_COMMANDS.to_vec();
    reversed.reverse();
    assert_eq!(explain(&d, doc, &reversed), a);
}

#[test]
fn suggestion_count_is_bounded() {
    let doc = "\\begin{itemize}\nx\n\ny\n\\begin{a}\\end{a}\n\\section{s}\n";
    let d = err(
        "unterminated environment 'itemize' — no matching \\end",
        doc,
        "\\begin{itemize}",
        "closed",
    );
    let x = explain(&d, doc, &[]);
    assert!(x.suggestions.len() <= flashtex_diagnostic_explanations::suggest::MAX_SUGGESTIONS);
    assert!(!x.suggestions.is_empty());
}

#[test]
fn arbitrary_spans_never_panic_and_edits_stay_applicable() {
    // Deterministic pseudo-random spans (including ones inside multi-byte
    // scalars, inverted, or past the end) against every catalog message.
    let doc = "\\begin{a}é{ü}^_$\\frac x}\n\n\\section ✓ [x\\end{b} % ü\n\\newcommand{\\a}{#1}\\a";
    let messages: Vec<String> = flashtex_diagnostic_explanations::catalog::ENTRIES
        .iter()
        .flat_map(|e| e.templates.iter().map(|t| t.replace('*', "a")))
        .collect();
    let mut seed: u64 = 0x9E3779B97F4A7C15;
    let mut next = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed % (doc.len() as u64 + 8)) as usize
    };
    for message in &messages {
        for _ in 0..40 {
            let (a, b) = (next(), next());
            let d = Diagnostic::at(Severity::Error, message, P, a, b, "r");
            let x = explain(&d, doc, DEFAULT_SUPPORTED_COMMANDS);
            assert!(!x.suggestions.is_empty());
            for s in &x.suggestions {
                if !s.edits.is_empty() {
                    preview::apply_edits(doc, &s.edits)
                        .unwrap_or_else(|e| panic!("{message} span {a}..{b}: {e} ({})", s.text));
                }
            }
            let _ = x.to_json();
        }
    }
}
