//! GH-497 (GH-TEXTSUPERSCRIPT): `\textsuperscript{text}` and
//! `\textsubscript{text}` errored `unsupported_feature` in text mode.
//! Both are kernel text commands (latex.ltx `ltmisc.dtx`
//! `\@textsuperscript` / `\@textsubscript`): the argument is set at the
//! `\sf@size` of the current size, raised or lowered like a math script
//! of an empty nucleus.
use flashtex_compiler::layout::layout;
use flashtex_compiler::parser::{parse, Block, Inline};

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
            Inline::TextScript(t) => out.push_str(&text_of(&t.content)),
            _ => {}
        }
    }
    out
}

fn scripts(inlines: &[Inline]) -> Vec<(bool, String)> {
    inlines
        .iter()
        .filter_map(|inline| match inline {
            Inline::TextScript(t) => Some((t.superscript, text_of(&t.content))),
            _ => None,
        })
        .collect()
}

/// Issue #497: `E=mc\textsuperscript{2} H\textsubscript{2}O` reported both
/// commands as `unsupported_feature`. Both are kernel commands now.
#[test]
fn text_scripts_typeset_with_no_diagnostics() {
    let source = "E=mc\\textsuperscript{2} H\\textsubscript{2}O";
    let parsed = parse(source);
    assert!(
        parsed.diagnostics.is_empty(),
        "text scripts should not diagnose: {:?}",
        parsed.diagnostics
    );
    let found = scripts(&paragraph_inlines(source));
    assert_eq!(
        found,
        vec![(true, "2".to_string()), (false, "2".to_string())],
        "both commands must parse into raised/lowered wrappers: {found:?}"
    );
}

/// The wrapper keeps its argument visible and the baseline returns to
/// normal afterward: only the argument is wrapped, surrounding text is
/// ordinary `Inline::Text`.
#[test]
fn text_script_wraps_only_its_argument() {
    let source = "a\\textsuperscript{b}c";
    let inlines = paragraph_inlines(source);
    let kinds: Vec<&str> = inlines
        .iter()
        .map(|inline| match inline {
            Inline::TextScript(t) if t.superscript => "super",
            Inline::TextScript(_) => "sub",
            Inline::Text { .. } => "text",
            _ => "other",
        })
        .collect();
    assert_eq!(kinds, vec!["text", "super", "text"]);
    assert_eq!(text_of(&inlines), "abc");
}

/// The compiler's own Core 14 layout sets the argument smaller and shifts
/// its baseline: up for `\textsuperscript`, down for `\textsubscript`.
#[test]
fn text_scripts_shift_the_baseline_in_opposite_directions() {
    let source = "a\\textsuperscript{b}c H\\textsubscript{2}O";
    let parsed = parse(source);
    assert!(parsed.diagnostics.is_empty());
    let pages = layout(&parsed.blocks);
    let items: Vec<_> = pages.iter().flat_map(|p| &p.items).collect();
    let baseline_of = |text: &str| {
        items
            .iter()
            .find(|item| item.text == text)
            .unwrap_or_else(|| panic!("expected item {text:?}"))
            .baseline_y_pt
    };
    let (body, raised, lowered) = (baseline_of("a"), baseline_of("b"), baseline_of("2"));
    assert!(
        raised < body,
        "superscript must sit above the baseline ({raised} < {body})"
    );
    assert!(
        lowered > body,
        "subscript must sit below the baseline ({lowered} > {body})"
    );
    assert_eq!(
        baseline_of("c"),
        body,
        "text after the superscript returns to the baseline"
    );
    let size_of = |text: &str| {
        items
            .iter()
            .find(|item| item.text == text)
            .unwrap_or_else(|| panic!("expected item {text:?}"))
            .font_size_pt
    };
    assert!(
        size_of("b") < size_of("a"),
        "superscript must be set smaller than body text"
    );
    assert!(
        size_of("2") < size_of("H"),
        "subscript must be set smaller than body text"
    );
}
