//! GH-SOUL-SO-HL (issue #502): soul `\so{text}` (letterspacing) and
//! `\hl{text}` (highlight) errored `unknown_command`.
//!
//! - `\so` inserts soul's `.14em` letterskip kern between every two adjacent
//!   letters of its argument, reusing the existing text-kern machinery, so
//!   the spaced run lays out measurably wider than the plain word.
//! - `\hl` is the argument on a yellow box: the exact xcolor `\colorbox`
//!   node (same paint path) with xcolor's own yellow fill.
//! - Both compose (`\hl{\so{..}}`, `\so{\hl{..}}`); both need
//!   `\usepackage{soul}` and otherwise diagnose while keeping the text.
//! - soul `\st` (strikethrough) is out of scope (issue #330's ulem-side
//!   work) and must keep its exact `unknown_command` error.
use flashtex_compiler::color::{ColorSpace, DeviceColor};
use flashtex_compiler::diagnostics::DiagnosticCode;
use flashtex_compiler::parser::{parse, Block, Inline};
use flashtex_compiler::text_builtins::TextDimen;

fn soul_doc(body: &str) -> String {
    format!(
        "\\documentclass{{article}}\n\\usepackage{{soul}}\n\\begin{{document}}\n{body}\n\\end{{document}}"
    )
}

fn paragraph_inlines(source: &str) -> Vec<Inline> {
    parse(source)
        .blocks
        .into_iter()
        .find_map(|block| match block {
            Block::Paragraph(inlines) => Some(inlines),
            _ => None,
        })
        .unwrap_or_default()
}

fn text_of(inlines: &[Inline]) -> String {
    let mut out = String::new();
    for inline in inlines {
        match inline {
            Inline::Text { text, .. } => out.push_str(text),
            Inline::ColorBox(b) => out.push_str(&text_of(&b.content)),
            Inline::Underline(u) => out.push_str(&text_of(&u.content)),
            _ => {}
        }
    }
    out
}

fn yellow() -> DeviceColor {
    DeviceColor::from_billionths(ColorSpace::Cmyk, &[0, 0, 1_000_000_000, 0])
        .expect("soul highlight yellow parses")
}

/// Issue #502: `\so{text}` is letterspaced, not unknown.
#[test]
fn so_with_soul_kerns_between_letters() {
    let source = soul_doc("Text \\so{text} here.");
    let diagnostics = parse(&source).diagnostics;
    assert!(
        diagnostics.is_empty(),
        "\\so with soul loaded should be silent: {diagnostics:?}"
    );
    let inlines = paragraph_inlines(&source);
    let joined = text_of(&inlines);
    assert!(
        joined.contains("text"),
        "argument must stay visible: {inlines:?}"
    );
    let want = TextDimen::parse("0.14em").expect("letterskip parses");
    let mut letters = 0;
    let mut kerns = 0;
    for inline in &inlines {
        match inline {
            Inline::Text { text, .. } if text == "t" || text == "e" || text == "x" => {
                letters += 1
            }
            Inline::Kern { amount, .. } => {
                assert_eq!(amount, &want, "soul letterskip is 0.14em: {inlines:?}");
                kerns += 1;
            }
            _ => {}
        }
    }
    assert!(letters >= 4, "letters split apart: {inlines:?}");
    assert_eq!(kerns, 3, "one kern between every two letters: {inlines:?}");
}

/// `\so` without soul diagnoses (like ulem without ulem) and keeps text.
#[test]
fn so_without_soul_diagnoses_and_keeps_text() {
    let source = "\\documentclass{article}\n\\begin{document}\nText \\so{text} here.\n\\end{document}";
    let diagnostics = parse(source).diagnostics;
    assert!(
        diagnostics
            .iter()
            .any(|d| d.message.contains("\\so needs \\usepackage{soul}")),
        "missing-package diagnostic: {diagnostics:?}"
    );
    assert!(
        !diagnostics.iter().any(|d| d.code == Some(DiagnosticCode::UnknownCommand)),
        "\\so is known once implemented: {diagnostics:?}"
    );
    let joined = text_of(&paragraph_inlines(source));
    assert!(joined.contains("text"), "argument kept: {joined:?}");
}

/// Issue #502: `\hl{text}` is a yellow colorbox, not unknown.
#[test]
fn hl_with_soul_paints_a_yellow_colorbox() {
    let source = soul_doc("Text \\hl{text} here.");
    let diagnostics = parse(&source).diagnostics;
    assert!(
        diagnostics.is_empty(),
        "\\hl with soul loaded should be silent: {diagnostics:?}"
    );
    let inlines = paragraph_inlines(&source);
    let boxes: Vec<_> = inlines
        .iter()
        .filter_map(|inline| match inline {
            Inline::ColorBox(b) => Some(b),
            _ => None,
        })
        .collect();
    assert_eq!(boxes.len(), 1, "one highlight box: {inlines:?}");
    assert_eq!(boxes[0].fill, yellow(), "highlight fill is yellow");
    assert_eq!(boxes[0].frame, None, "highlight has no frame");
    assert_eq!(text_of(&boxes[0].content), "text");
}

/// `\hl` without soul diagnoses and keeps text.
#[test]
fn hl_without_soul_diagnoses_and_keeps_text() {
    let source = "\\documentclass{article}\n\\begin{document}\nText \\hl{text} here.\n\\end{document}";
    let diagnostics = parse(source).diagnostics;
    assert!(
        diagnostics
            .iter()
            .any(|d| d.message.contains("\\hl needs \\usepackage{soul}")),
        "missing-package diagnostic: {diagnostics:?}"
    );
    let joined = text_of(&paragraph_inlines(source));
    assert!(joined.contains("text"), "argument kept: {joined:?}");
}

/// Issue #502: `\hl{\so{..}}` and `\so{\hl{..}}` both work.
#[test]
fn so_and_hl_compose_both_ways() {
    let outer_hl = soul_doc("Text \\hl{\\so{ab}} here.");
    let diagnostics = parse(&outer_hl).diagnostics;
    assert!(diagnostics.is_empty(), "\\hl{{\\so}} silent: {diagnostics:?}");
    let inlines = paragraph_inlines(&outer_hl);
    let boxed = inlines.iter().find_map(|inline| match inline {
        Inline::ColorBox(b) => Some(b),
        _ => None,
    });
    let boxed = boxed.expect("highlight box survives");
    assert_eq!(boxed.fill, yellow());
    assert!(
        boxed.content.iter().any(|i| matches!(i, Inline::Kern { .. })),
        "letterspacing inside the highlight: {:?}",
        boxed.content
    );

    let outer_so = soul_doc("Text \\so{\\hl{ab}} here.");
    let diagnostics = parse(&outer_so).diagnostics;
    assert!(diagnostics.is_empty(), "\\so{{\\hl}} silent: {diagnostics:?}");
    let inlines = paragraph_inlines(&outer_so);
    assert!(
        inlines.iter().any(|i| matches!(i, Inline::ColorBox(_))),
        "highlight box inside the spacing: {inlines:?}"
    );
    assert_eq!(text_of(&inlines).replace(' ', ""), "Textabhere.");
}

/// Out of scope: soul `\st` keeps its exact `unknown_command` error, with
/// or without the package.
#[test]
fn st_still_errors_unknown_command() {
    for source in [
        soul_doc("Text \\st{text} here."),
        "\\documentclass{article}\n\\begin{document}\nText \\st{text} here.\n\\end{document}".to_string(),
    ] {
        let diagnostics = parse(&source).diagnostics;
        assert!(
            diagnostics.iter().any(|d| d.code == Some(DiagnosticCode::UnknownCommand)
                && d.message.contains("\\st")),
            "\\st must stay unknown_command: {diagnostics:?}"
        );
    }
}

/// `\usepackage{soul}` loads silently (no "not implemented" warning).
#[test]
fn soul_package_load_is_silent() {
    let source = soul_doc("Text here.");
    let diagnostics = parse(&source).diagnostics;
    assert!(
        !diagnostics
            .iter()
            .any(|d| d.message.contains("not implemented")),
        "soul is implemented: {diagnostics:?}"
    );
}
