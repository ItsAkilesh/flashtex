//! fancyhdr (v5.2) directives as structured blocks: field selectors, field
//! content with page marks, the legacy `\lhead` family, offsets,
//! `\fancypagestyle` bodies and the rule/mark macro snapshot the expansion
//! pass takes at `\begin{document}`.

use flashtex_compiler::parser::{parse, Block, Inline, PageMarkKind, PageSlot, RuleMacro, SlotPosition};

fn doc(preamble: &str, body: &str) -> String {
    format!("\\documentclass{{article}}\n\\usepackage{{fancyhdr}}\n{preamble}\n\\begin{{document}}\n{body}\n\\end{{document}}\n")
}

fn fields(blocks: &[Block]) -> Vec<(&Vec<PageSlot>, &Vec<Inline>)> {
    blocks
        .iter()
        .filter_map(|b| match b {
            Block::PageField { slots, content, .. } => Some((slots, content)),
            _ => None,
        })
        .collect()
}

fn slot(even: bool, position: SlotPosition, footer: bool) -> PageSlot {
    PageSlot { even, position, footer }
}

fn errors(parsed: &flashtex_compiler::parser::Parsed) -> Vec<String> {
    parsed.diagnostics.iter().filter(|d| d.severity == flashtex_compiler::diagnostics::Severity::Error).map(|d| d.message.clone()).collect()
}

#[test]
fn fancyhead_selectors_cross_sides_fields_and_lines() {
    let p = parse(&doc("\\pagestyle{fancy}\n\\fancyhead[LE,RO]{\\thepage}\n\\fancyfoot{x}\n\\fancyhf[C]{c}", "Body."));
    assert!(errors(&p).is_empty(), "{:?}", errors(&p));
    let f = fields(&p.blocks);
    assert_eq!(f.len(), 3);
    assert_eq!(f[0].0, &vec![slot(true, SlotPosition::Left, false), slot(false, SlotPosition::Right, false)]);
    // `\fancyfoot{x}`: both sides, all three fields, footer only.
    assert_eq!(f[1].0.len(), 6);
    assert!(f[1].0.iter().all(|s| s.footer));
    // `\fancyhf[C]`: both sides, centre, header and footer.
    assert_eq!(
        f[2].0,
        &vec![
            slot(true, SlotPosition::Center, false),
            slot(true, SlotPosition::Center, true),
            slot(false, SlotPosition::Center, false),
            slot(false, SlotPosition::Center, true)
        ]
    );
}

#[test]
fn fancyhf_empty_clears_all_twelve_slots() {
    let p = parse(&doc("\\fancyhf{}", "Body."));
    let f = fields(&p.blocks);
    assert_eq!(f[0].0.len(), 12);
    assert!(f[0].1.is_empty());
}

#[test]
fn field_content_carries_page_marks_styles_and_line_breaks() {
    let p = parse(&doc(
        "\\fancyhead[L]{Page \\thepage\\ of \\textbf{many}\\\\second}\n\\fancyhead[R]{\\nouppercase{\\leftmark} \\rightmark \\thesection}",
        "Body.",
    ));
    assert!(errors(&p).is_empty(), "{:?}", errors(&p));
    let f = fields(&p.blocks);
    let left = f[0].1;
    assert!(matches!(&left[0], Inline::Text { text, .. } if text == "Page"));
    assert!(matches!(&left[1], Inline::PageMark { kind: PageMarkKind::PageNumber, uppercase: true, .. }));
    assert!(left.iter().any(|i| matches!(i, Inline::Text { text, style, .. } if text == "many" && style.bold)));
    assert!(left.iter().any(|i| matches!(i, Inline::LineBreak { .. })));
    assert!(matches!(left.last(), Some(Inline::Text { text, .. }) if text == "second"));
    let right = f[1].1;
    assert!(matches!(&right[0], Inline::PageMark { kind: PageMarkKind::LeftMark, uppercase: false, .. }));
    assert!(matches!(&right[1], Inline::PageMark { kind: PageMarkKind::RightMark, uppercase: true, .. }));
    assert!(matches!(&right[2], Inline::PageMark { kind: PageMarkKind::Section, .. }));
}

#[test]
fn legacy_lhead_sets_both_sides_unless_given_an_even_version() {
    let p = parse(&doc("\\lhead{Title}\n\\rfoot[\\thepage]{Odd}", "Body."));
    assert!(errors(&p).is_empty(), "{:?}", errors(&p));
    let f = fields(&p.blocks);
    assert_eq!(f[0].0, &vec![slot(false, SlotPosition::Left, false), slot(true, SlotPosition::Left, false)]);
    assert_eq!(f[1].0, &vec![slot(false, SlotPosition::Right, true)]);
    assert!(matches!(&f[1].1[0], Inline::Text { text, .. } if text == "Odd"));
    assert_eq!(f[2].0, &vec![slot(true, SlotPosition::Right, true)]);
    assert!(matches!(&f[2].1[0], Inline::PageMark { kind: PageMarkKind::PageNumber, .. }));
}

#[test]
fn rule_widths_and_mark_macros_are_snapshotted_at_begin_document() {
    let p = parse(&doc(
        "\\renewcommand{\\headrulewidth}{0pt}\\renewcommand{\\footrulewidth}{1pt}\\renewcommand{\\headrule}{}\n\\renewcommand{\\sectionmark}[1]{\\markright{\\thesection.\\ #1}}",
        "Body.",
    ));
    assert!(errors(&p).is_empty(), "{:?}", errors(&p));
    let state = p.blocks.iter().find_map(|b| match b {
        Block::PageStyleState { head_rule_width_pt, foot_rule_width_pt, foot_rule_skip_pt, head_rule, foot_rule, section_mark, chapter_mark, .. } => {
            Some((*head_rule_width_pt, *foot_rule_width_pt, *foot_rule_skip_pt, head_rule.clone(), foot_rule.clone(), section_mark.clone(), chapter_mark.clone()))
        }
        _ => None,
    });
    let (hw, fw, fs, hr, fr, sm, cm) = state.expect("state block");
    assert_eq!(hw, Some(0.0));
    assert_eq!(fw, Some(1.0));
    // `.3\normalbaselineskip` is not a plain dimension: the layout's default.
    assert_eq!(fs, None);
    assert_eq!(hr, RuleMacro::Empty);
    assert_eq!(fr, RuleMacro::Default);
    assert_eq!(sm.as_deref(), Some("\\long macro:#1->\\markright {\\thesection .\\ #1}"));
    assert_eq!(cm, None);
}

#[test]
fn defaults_are_the_package_defaults() {
    let p = parse(&doc("\\pagestyle{fancy}", "Body."));
    let state = p.blocks.iter().find_map(|b| match b {
        Block::PageStyleState { head_rule_width_pt, foot_rule_width_pt, head_rule_skip_pt, .. } => Some((*head_rule_width_pt, *foot_rule_width_pt, *head_rule_skip_pt)),
        _ => None,
    });
    assert_eq!(state, Some((Some(0.4), Some(0.0), Some(0.0))));
}

#[test]
fn fancypagestyle_body_is_nested_with_its_own_local_rule_width() {
    let p = parse(&doc(
        "\\fancypagestyle{plain}{\\fancyhf{}\\renewcommand{\\headrulewidth}{2pt}\\fancyfoot[C]{\\thepage}}\n\\fancypagestyle*{other}[plain]{\\fancyhead[C]{x}}",
        "Body.",
    ));
    assert!(errors(&p).is_empty(), "{:?}", errors(&p));
    let defs: Vec<_> = p
        .blocks
        .iter()
        .filter_map(|b| match b {
            Block::PageStyleDefinition { name, base, starred, body, .. } => Some((name.as_str(), base.as_str(), *starred, body)),
            _ => None,
        })
        .collect();
    assert_eq!(defs.len(), 2);
    assert_eq!((defs[0].0, defs[0].1, defs[0].2), ("plain", "fancy", false));
    assert_eq!((defs[1].0, defs[1].1, defs[1].2), ("other", "plain", true));
    let body = defs[0].3;
    assert!(matches!(&body[0], Block::PageField { slots, content, .. } if slots.len() == 12 && content.is_empty()));
    assert!(matches!(&body[1], Block::PageField { slots, .. } if slots.len() == 2));
    assert!(matches!(body.last(), Some(Block::PageStyleState { head_rule_width_pt: Some(w), .. }) if *w == 2.0));
    // The document-level snapshot keeps the outer value.
    let outer = p.blocks.iter().find_map(|b| match b {
        Block::PageStyleState { head_rule_width_pt, .. } => *head_rule_width_pt,
        _ => None,
    });
    assert_eq!(outer, Some(0.4));
}

#[test]
fn offsets_keep_the_dimension_text_for_the_layout() {
    let p = parse(&doc("\\fancyhfoffset[L]{1in}\\fancyheadoffset[RO]{\\marginparsep+\\marginparwidth}", "Body."));
    assert!(errors(&p).is_empty(), "{:?}", errors(&p));
    let offsets: Vec<_> = p
        .blocks
        .iter()
        .filter_map(|b| match b {
            Block::PageFieldOffset { slots, dimen, .. } => Some((slots.len(), dimen.as_str())),
            _ => None,
        })
        .collect();
    assert_eq!(offsets, vec![(4, "1in"), (1, "\\marginparsep +\\marginparwidth")]);
}

#[test]
fn illegal_selector_letters_are_the_package_error() {
    let p = parse(&doc("\\fancyhead[X]{x}\\fancyheadoffset[C]{1pt}", "Body."));
    let e = errors(&p);
    assert!(e.iter().any(|m| m.contains("Illegal char `x' in \\fancyhead argument: [X]")), "{e:?}");
    assert!(e.iter().any(|m| m.contains("Illegal char `c' in \\fancyheadoffset argument: [C]")), "{e:?}");
    assert!(fields(&p.blocks).is_empty());
}

#[test]
fn documents_without_fancyhdr_get_no_state_block_and_pagestyle_is_accepted_in_the_preamble() {
    let p = parse("\\documentclass{article}\n\\pagestyle{headings}\n\\begin{document}\nBody.\n\\end{document}\n");
    assert!(errors(&p).is_empty(), "{:?}", errors(&p));
    assert!(!p.blocks.iter().any(|b| matches!(b, Block::PageStyleState { .. })));
    assert_eq!(p.blocks.len(), 1);
}

#[test]
fn fields_in_the_body_keep_their_position_among_paragraphs() {
    let p = parse(&doc("\\pagestyle{fancy}", "First.\n\n\\fancyhead[C]{Second part}\n\nSecond."));
    let kinds: Vec<&str> = p
        .blocks
        .iter()
        .map(|b| match b {
            Block::Paragraph(_) => "para",
            Block::PageField { .. } => "field",
            Block::PageStyleState { .. } => "state",
            _ => "other",
        })
        .collect();
    assert_eq!(kinds, vec!["state", "para", "field", "para"]);
}
