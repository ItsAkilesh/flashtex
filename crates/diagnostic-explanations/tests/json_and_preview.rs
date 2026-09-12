//! JSON round-trip, compile_result ingestion, and fix previews.

use flashtex_diagnostic_explanations::preview::{self, Rebase};
use flashtex_diagnostic_explanations::{
    Category, Confidence, DEFAULT_SUPPORTED_COMMANDS, Diagnostic, Document, Edit, Explanation,
    Severity, Suggestion, explain, explain_compile_result_json, json,
};

fn sample() -> (String, Diagnostic) {
    let doc = "Line \"one\"\n\\begin{itemize}\nitem ü\t✓\n".to_string();
    let s = doc.find("\\begin").unwrap();
    let d = Diagnostic::at(
        Severity::Error,
        "unterminated environment 'itemize' — no matching \\end",
        "main.tex",
        s,
        s + "\\begin{itemize}".len(),
        "closed the environment at end of input",
    );
    (doc, d)
}

#[test]
fn explanation_json_round_trips_exactly() {
    let (doc, d) = sample();
    let x = explain(&d, &doc, DEFAULT_SUPPORTED_COMMANDS);
    let j = x.to_json();
    let back = Explanation::from_json(&j).unwrap();
    assert_eq!(back, x);
    assert_eq!(back.to_json(), j);
    // Fixed key order the Mac decoder can rely on.
    assert!(j.starts_with("{\"catalog_id\":\"unterminated-environment\",\"title\":"));
    assert!(j.contains("\"context\":{\"path\":\"main.tex\""));
    // Control characters and quotes are escaped; non-ASCII passes through.
    assert!(j.contains("\\t✓"));
    assert!(j.contains("Line \\\"one\\\""));
}

#[test]
fn fallback_and_no_context_round_trip() {
    let d = Diagnostic::new(Severity::Warning, "brand new wording", None, None);
    let x = explain(&d, "", &[]);
    assert_eq!(x.category, Category::Unknown);
    let j = x.to_json();
    assert!(j.contains("\"catalog_id\":null"));
    assert!(j.ends_with("\"context\":null}"));
    assert_eq!(Explanation::from_json(&j).unwrap(), x);
}

#[test]
fn batch_json_is_an_array() {
    let (doc, d) = sample();
    let xs = vec![explain(&d, &doc, &[]), explain(&d, &doc, &[])];
    let j = json::explanations_to_json(&xs);
    let v = json::parse(&j).unwrap();
    assert_eq!(v.as_arr().unwrap().len(), 2);
    let first = json::explanation_from_value(&v.as_arr().unwrap()[0]).unwrap();
    assert_eq!(first, xs[0]);
}

#[test]
fn compile_result_envelope_and_payload_are_both_accepted() {
    // Shape from docs/contracts/runtime-v1.md and crates/compiler/src/diagnostics.rs::to_json.
    let payload = r#"{"project_id":"p","revision":3,"status":"recovered","pages":[],
        "diagnostics":[
          {"severity":"error","message":"unmatched '}' — no group is open here",
           "source":{"path":"main.tex","start_byte":4,"end_byte":5},
           "recovery":"ignored the stray brace and continued"},
          {"severity":"warning","message":"2 documents were supplied; this version compiles only the entry document",
           "source":null,"recovery":"compiled the entry document alone"}
        ],"pdf_path":null}"#;
    let envelope = format!(
        r#"{{"protocol_version":1,"id":"c1","type":"compile_result","payload":{}}}"#,
        payload
    );
    let docs = [Document {
        path: "main.tex",
        text: "abc }x",
    }];
    for input in [payload, envelope.as_str()] {
        let xs = explain_compile_result_json(input, &docs, DEFAULT_SUPPORTED_COMMANDS).unwrap();
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].catalog_id.as_deref(), Some("stray-close-brace"));
        assert_eq!(
            xs[0].suggestions[0].edits,
            vec![Edit::delete("main.tex", 4, 5)]
        );
        assert_eq!(xs[1].category, Category::MultiFile);
        assert!(xs[1].context.is_none());
    }
    assert!(explain_compile_result_json("{\"pages\":[]}", &docs, &[]).is_err());
    assert!(explain_compile_result_json("not json", &docs, &[]).is_err());
    let bad_sev =
        r#"{"diagnostics":[{"severity":"fatal","message":"x","source":null,"recovery":null}]}"#;
    assert!(explain_compile_result_json(bad_sev, &docs, &[]).is_err());
}

// ---- previews ------------------------------------------------------------------

#[test]
fn preview_reports_excerpts_and_rebased_span() {
    let doc = "Intro ü\n\\sectoin{Title}\nbody line\nlast";
    let s = doc.find("\\sectoin").unwrap();
    let d = Diagnostic::at(
        Severity::Error,
        "\\sectoin is not supported by this compiler version",
        "main.tex",
        s,
        s + 8,
        "skipped",
    );
    let x = explain(&d, doc, DEFAULT_SUPPORTED_COMMANDS);
    let fix = &x.suggestions[0];
    let p = preview::preview(doc, fix, Some((s, s + 8))).unwrap();
    assert_eq!(p.after_text, "Intro ü\n\\section{Title}\nbody line\nlast");
    assert_eq!(p.before_excerpt, "\\sectoin{Title}");
    assert_eq!(p.after_excerpt, "\\section{Title}");
    assert_eq!(p.before_excerpt_range, (s, s + "\\sectoin{Title}".len()));
    // The diagnostic's own span was replaced: it cannot survive the fix.
    assert_eq!(p.original_span_after, None);
    assert!(!p.span_still_valid);
    // The Mac-style changed region covers just the differing bytes.
    assert_eq!(&doc[p.changed.start_byte..p.changed.old_end_byte], "oi");
    assert_eq!(p.changed.replacement, "io");
}

#[test]
fn preview_keeps_a_span_before_the_edit_and_shifts_one_after() {
    let doc = "aa {bb\n\ncc }dd";
    // Unclosed '{' at 3; stray '}' at 11 (two diagnostics in one document).
    let open = Diagnostic::at(
        Severity::Error,
        "unmatched '{' — group never closed",
        "m",
        3,
        4,
        "treated the rest of the document as part of the group",
    );
    let x = explain(&open, doc, &[]);
    let fix = x
        .suggestions
        .iter()
        .find(|s| s.text.contains("paragraph break"))
        .unwrap();
    let p = preview::preview(doc, fix, Some((3, 4))).unwrap();
    assert_eq!(p.after_text, "aa {bb}\n\ncc }dd");
    assert_eq!(p.original_span_after, Some((3, 4)));
    assert!(p.span_still_valid);
    // A span after the insertion shifts by the inserted length, like SourceMapping.
    assert_eq!(
        preview::rebase(11, 12, doc, &p.after_text),
        Rebase::Rebased { start: 12, end: 13 }
    );
    assert_eq!(&p.after_text[12..13], "}");
}

#[test]
fn preview_of_multi_edit_suggestion_on_multibyte_text() {
    let doc = "x_ü and more";
    let d = Diagnostic::at(
        Severity::Error,
        "math script marker used outside math mode",
        "m",
        1,
        2,
        "ignored the script marker and continued",
    );
    let x = explain(&d, doc, &[]);
    let wrap = &x.suggestions[0];
    assert_eq!(wrap.edits.len(), 2);
    let p = preview::preview(doc, wrap, Some((1, 2))).unwrap();
    assert_eq!(p.after_text, "$x_ü$ and more");
    assert_eq!(p.before_excerpt, "x_ü and more");
    assert_eq!(p.after_excerpt, "$x_ü$ and more");
    // The span lies inside the collapsed changed region (prefix/suffix rule).
    assert_eq!(p.original_span_after, None);
}

#[test]
fn preview_refuses_advice_and_bad_edits() {
    let advice = Suggestion::advice("do something", Confidence::Low);
    assert_eq!(
        preview::preview("abc", &advice, None),
        Err(preview::PreviewError::NoEdits)
    );
    let inside_scalar = Suggestion::new("bad", vec![Edit::insert("m", 1, "!")], Confidence::Low);
    assert!(matches!(
        preview::preview("é", &inside_scalar, None),
        Err(preview::PreviewError::OutOfBounds {
            start_byte: 1,
            end_byte: 1
        })
    ));
    let overlapping = Suggestion::new(
        "bad",
        vec![Edit::delete("m", 0, 2), Edit::delete("m", 1, 3)],
        Confidence::Low,
    );
    assert_eq!(
        preview::preview("abcd", &overlapping, None),
        Err(preview::PreviewError::Overlap)
    );
}

#[test]
fn every_generated_edit_is_previewable() {
    // Every suggestion the crate produces for these documents must apply
    // cleanly: in-bounds, on scalar boundaries, non-overlapping.
    let cases: &[(&str, &str, &str)] = &[
        ("unmatched '{' — group never closed", "é {ü\n\nx", "{"),
        ("unmatched '}' — no group is open here", "é }ü", "}"),
        (
            "unterminated environment 'itemize' — no matching \\end",
            "\\begin{itemize}\nü\n\\section{é}\n",
            "\\begin{itemize}",
        ),
        (
            "\\end{b} does not match \\begin{a}",
            "\\begin{a}ü\\end{b}",
            "\\end",
        ),
        ("\\end{a} with no matching \\begin", "ü\\end{a}", "\\end"),
        (
            "environment 'center' is not implemented; its body is typeset as plain text",
            "\\begin{center}ü\\end{center}",
            "\\begin",
        ),
        (
            "math mode is not implemented in this version",
            "a $ü$ b",
            "$",
        ),
        (
            "\\sectoin is not supported by this compiler version",
            "\\sectoin{ü}",
            "\\sectoin",
        ),
        ("\\emph requires a braced argument", "\\emph ü", "\\emph"),
        (
            "argument to \\emph is missing its closing brace",
            "\\emph{ü",
            "{",
        ),
        ("\\emph was given an empty argument", "\\emph{}", "\\emph{}"),
        ("unmatched '}' in math mode", "$ü}$", "}"),
        ("duplicate script on a math atom", "$x^ü^2$", "^2"),
        ("script marker has no preceding math atom", "$^ü$", "^"),
        ("math group is missing its closing brace", "${ü$", "ü"),
        ("math script is missing its argument", "$x^$", "^"),
        ("unexpected math delimiter inside math mode", "$a$b$", "$b"),
        (
            "\\alhpa is not supported in math mode",
            "$\\alhpa$",
            "\\alhpa",
        ),
        (
            "\\frac requires a braced math argument",
            "$\\frac ü$",
            "\\frac",
        ),
        ("stray \\] has no matching \\[", "ü \\]", "\\]"),
        ("math script marker used outside math mode", "ü_2", "_"),
        (
            "display math is missing its closing delimiter",
            "\\[ ü\n\nx",
            "\\[",
        ),
        ("inline math is missing its closing '$'", "$ ü\n\nx", "$"),
        (
            "\\documentclass was given an empty argument",
            "\\documentclass{}",
            "\\documentclass",
        ),
        (
            "\\usepackage was given an empty package list",
            "\\usepackage{}",
            "\\usepackage{}",
        ),
        (
            "packages ü are recognised but not implemented",
            "\\usepackage{ü}\nx",
            "\\usepackage{ü}",
        ),
        (
            "\\newcommand argument count must be an integer from 0 to 9",
            "\\newcommand{\\a}[x]{ü}",
            "[x]",
        ),
        (
            "\\newcommand cannot redefine existing command \\a",
            "\\newcommand{\\a}{ü}",
            "\\newcommand{\\a}",
        ),
        (
            "\\renewcommand cannot redefine undefined command \\a",
            "\\renewcommand{\\a}{ü}",
            "\\renewcommand{\\a}",
        ),
        (
            "macro \\a exceeded the expansion recursion limit of 64",
            "\\a ü",
            "\\a",
        ),
        (
            "macro replacement references #1 but that argument is not declared",
            "\\newcommand{\\a}{#1}\\a ü",
            "\\a ü",
        ),
        (
            "optional argument is missing its closing ']'",
            "\\documentclass[ü\n{x}",
            "[ü",
        ),
        (
            "\\section is not supported in the document preamble",
            "\\section{ü}\n\\begin{document}x\\end{document}",
            "\\section",
        ),
    ];
    for (message, doc, needle) in cases {
        let s = doc.find(needle).unwrap();
        let d = Diagnostic::at(
            Severity::Error,
            message,
            "m",
            s,
            s + needle.len(),
            "recovered",
        );
        let x = explain(&d, doc, DEFAULT_SUPPORTED_COMMANDS);
        assert!(x.catalog_id.is_some(), "{message:?} should be catalogued");
        for sug in &x.suggestions {
            if sug.edits.is_empty() {
                continue;
            }
            let p = preview::preview(doc, sug, Some((s, s + needle.len())))
                .unwrap_or_else(|e| panic!("{message:?} / {:?}: {e}", sug.text));
            assert!(p.after_text.is_char_boundary(p.after_excerpt_range.0));
        }
    }
}
