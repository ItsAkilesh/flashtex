//! LaTeX box commands parse into structured `Inline::Box` /
//! `Inline::SetLength` / `Inline::LengthGlue` nodes (see `src/boxes.rs`);
//! geometry is the typesetter's (render-pipeline with `crates/tex-boxes`).

use flashtex_compiler::boxes::{BoxUnit, LengthValue, MeasuredDimension, TextBoxKind};
use flashtex_compiler::parser::{self, Block, Inline, ParagraphStyle};

fn body(src: &str) -> (Vec<Inline>, Vec<String>) {
    let doc = format!("\\documentclass{{article}}\n\\begin{{document}}\n{src}\n\\end{{document}}\n");
    let parsed = parser::parse(&doc);
    let mut inlines = Vec::new();
    for block in parsed.blocks {
        if let Block::Paragraph(content) = block {
            inlines.extend(content);
        }
    }
    let diags = parsed.diagnostics.iter().map(|d| d.message.clone()).collect();
    (inlines, diags)
}

fn boxes(inlines: &[Inline]) -> Vec<&flashtex_compiler::boxes::TextBox> {
    inlines
        .iter()
        .filter_map(|i| match i {
            Inline::Box(b) => Some(&**b),
            _ => None,
        })
        .collect()
}

fn text_of(inlines: &[Inline]) -> String {
    let mut out = Vec::new();
    for i in inlines {
        match i {
            Inline::Text { text, .. } => out.push(text.clone()),
            Inline::Box(b) => out.push(format!("[{}]", b.inline_lists().iter().map(|l| text_of(l)).collect::<Vec<_>>().join("|"))),
            _ => {}
        }
    }
    out.join(" ")
}

#[test]
fn horizontal_boxes_keep_their_arguments_and_content() {
    let (inlines, diags) = body(
        "A \\mbox{b c} \\fbox{d} \\makebox[2cm][r]{e} \\framebox[1.5\\width]{f} \\raisebox{2pt}[3pt][1pt]{g} \\smash{h} \\llap{i}\\rlap{j} \\strut",
    );
    assert!(diags.is_empty(), "{diags:?}");
    let b = boxes(&inlines);
    assert_eq!(b.len(), 9);
    assert!(matches!(b[0].kind, TextBoxKind::Make { width: None, pos: None, frame: false }));
    assert_eq!(text_of(&b[0].content), "b c");
    assert!(matches!(b[1].kind, TextBoxKind::Make { width: None, frame: true, .. }));
    match &b[2].kind {
        TextBoxKind::Make { width: Some(w), pos: Some('r'), frame: false } => {
            assert_eq!((w.integer, w.unit.clone()), (2, BoxUnit::Physical("cm".into())));
        }
        other => panic!("{other:?}"),
    }
    match &b[3].kind {
        TextBoxKind::Make { width: Some(w), pos: None, frame: true } => {
            assert_eq!((w.integer, w.frac.as_slice(), w.unit.clone()), (1, &[5u8][..], BoxUnit::Width));
        }
        other => panic!("{other:?}"),
    }
    match &b[4].kind {
        TextBoxKind::Raise { lift, height: Some(h), depth: Some(d) } => {
            assert_eq!((lift.integer, h.integer, d.integer), (2, 3, 1));
        }
        other => panic!("{other:?}"),
    }
    assert!(matches!(b[5].kind, TextBoxKind::Smash));
    assert!(matches!(b[6].kind, TextBoxKind::Lap { left: true }));
    assert!(matches!(b[7].kind, TextBoxKind::Lap { left: false }));
    assert!(matches!(b[8].kind, TextBoxKind::Strut));
    assert_eq!(text_of(&inlines), "A [b c] [d] [e] [f] [g] [h] [i] [j] []");
}

#[test]
fn phantoms_record_which_dimensions_they_keep() {
    let (inlines, diags) = body("\\phantom{x}\\hphantom{y}\\vphantom{z}");
    assert!(diags.is_empty(), "{diags:?}");
    let kinds: Vec<_> = boxes(&inlines).iter().map(|b| b.kind.clone()).collect();
    assert_eq!(
        kinds,
        vec![
            TextBoxKind::Phantom { horizontal: true, vertical: true },
            TextBoxKind::Phantom { horizontal: true, vertical: false },
            TextBoxKind::Phantom { horizontal: false, vertical: true },
        ]
    );
}

#[test]
fn parbox_and_minipage_keep_paragraphs_and_alignment() {
    let (inlines, diags) = body(
        "x \\parbox[t]{3cm}{one two\n\nthree} y \\begin{minipage}[b][2cm][s]{.4\\textwidth}\\centering mid\\end{minipage} z",
    );
    assert!(diags.is_empty(), "{diags:?}");
    let b = boxes(&inlines);
    assert_eq!(b.len(), 2);
    match &b[0].kind {
        TextBoxKind::Par { pos: Some('t'), height: None, inner: None, width, minipage: false } => {
            assert_eq!(width.unit, BoxUnit::Physical("cm".into()));
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(b[0].paragraphs.len(), 2);
    assert_eq!(text_of(&b[0].paragraphs[1].content), "three");
    match &b[1].kind {
        TextBoxKind::Par { pos: Some('b'), height: Some(h), inner: Some('s'), width, minipage: true } => {
            assert_eq!(h.integer, 2);
            assert_eq!(width.unit, BoxUnit::Length("textwidth".into()));
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(b[1].paragraphs.len(), 1);
    assert_eq!(b[1].paragraphs[0].style, Some(ParagraphStyle::Center));
    assert_eq!(text_of(&inlines), "x [|one two|three] y [|mid] z");
}

#[test]
fn saved_boxes_are_reused_and_lengths_measured() {
    let (inlines, diags) = body(
        "\\newsavebox{\\keep}\\newlength{\\w}\\sbox{\\keep}{kept}\\begin{lrbox}{\\keep}lr kept\\end{lrbox}\\usebox{\\keep} and \\usebox{\\keep}\\settowidth{\\w}{abc}\\hspace{\\w}q\\hspace{.5\\w}\\setlength{\\w}{2pt}",
    );
    assert!(diags.is_empty(), "{diags:?}");
    let b = boxes(&inlines);
    assert_eq!(b.len(), 2);
    assert_eq!(text_of(&b[0].content), "lr kept");
    assert_eq!(b[0].content, b[1].content);
    assert_ne!(b[0].span, b[1].span);
    let sets: Vec<_> = inlines
        .iter()
        .filter_map(|i| match i {
            Inline::SetLength(a) => Some(a),
            _ => None,
        })
        .collect();
    assert_eq!(sets.len(), 2);
    assert!(matches!(&sets[0].value, LengthValue::Measure { which: MeasuredDimension::Width, .. }));
    assert!(matches!(&sets[1].value, LengthValue::Dimen(d) if d.integer == 2));
    let glue: Vec<_> = inlines
        .iter()
        .filter_map(|i| match i {
            Inline::LengthGlue { dimen, .. } => Some(dimen),
            _ => None,
        })
        .collect();
    assert_eq!(glue.len(), 2);
    assert_eq!(glue[1].frac, vec![5]);
    assert_eq!(glue[1].unit, BoxUnit::Length("w".into()));
}

#[test]
fn undeclared_registers_are_diagnosed() {
    let (_, diags) = body("\\usebox{\\nothere}\\settowidth{\\nolen}{x}\\makebox[\\nolen]{y}");
    assert_eq!(diags.len(), 3, "{diags:?}");
    assert!(diags.iter().all(|d| d.contains("\\nothere") || d.contains("\\nolen")), "{diags:?}");
}

#[test]
fn nested_boxes_and_math_parse_inside_content() {
    let (inlines, diags) = body("\\fbox{$x^2$ and \\mbox{\\textbf{in}}}");
    assert!(diags.is_empty(), "{diags:?}");
    let b = boxes(&inlines);
    assert_eq!(b.len(), 1);
    assert!(b[0].content.iter().any(|i| matches!(i, Inline::Math { .. })));
    assert_eq!(boxes(&b[0].content).len(), 1);
}
