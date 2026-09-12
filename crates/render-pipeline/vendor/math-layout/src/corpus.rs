//! The declared math visual corpus (`fixtures/visual/*.tex`) and its layout
//! as single lines on a US-letter page.
//!
//! Every case pairs a fixture body (the declared LaTeX, included verbatim)
//! with the math list that body denotes. This is a lookup over declared
//! cases, **not** a LaTeX parser: the corpus compiler recognises bodies
//! byte-for-byte and refuses anything else.

use crate::boxes::{PositionedRuns, positioned_runs};
use crate::cm::CmMathMetrics;
use crate::fixtures;
use crate::layout::{Limitation, layout_with_report};
use crate::mathlist::{Atom, Limits, MathList};
use crate::metrics::{MathFontMetrics, SizeClass};
use crate::style::Style;

/// One piece of a corpus line.
#[derive(Debug, Clone)]
pub enum Segment {
    /// A word of roman text (no kerning or ligatures applied).
    Word(String),
    /// A math formula in the given style.
    Math(MathList, Style),
}

#[derive(Debug, Clone)]
pub struct Case {
    pub id: &'static str,
    /// The fixture file's body from `\begin{document}` to `\end{document}`.
    pub body: &'static str,
    pub segments: Vec<Segment>,
}

/// TeX points per PDF point.
pub const PT_PER_BP: f64 = 72.27 / 72.0;
/// US letter in PDF points.
pub const PAGE_WIDTH_BP: f64 = 612.0;
pub const PAGE_HEIGHT_BP: f64 = 792.0;
/// `geometry` `margin=1in`.
pub const MARGIN_PT: f64 = 72.27;
/// `\topskip` of the 12pt article class.
pub const TOPSKIP_PT: f64 = 12.0;

macro_rules! body {
    ($name:literal) => {
        include_str!(concat!("../fixtures/visual/", $name, ".tex"))
    };
}

fn strip_preamble(tex: &'static str) -> &'static str {
    let start = tex.find("\\begin{document}").unwrap_or(0);
    &tex[start..]
}

fn sym(s: &str) -> MathList {
    MathList::symbols(s)
}

fn display(list: MathList) -> Vec<Segment> {
    vec![Segment::Math(list, Style::DISPLAY)]
}

fn text(list: MathList) -> Vec<Segment> {
    vec![Segment::Math(list, Style::TEXT)]
}

/// All corpus cases in fixture order.
pub fn cases() -> Vec<Case> {
    let mut a = fixtures::stacked_fraction();
    a.atoms.push(Atom::symbol('='));
    a.atoms.push(Atom::symbol('1'));
    let mut b = fixtures::sqrt_x();
    b.atoms.push(Atom::symbol('+'));
    b.atoms.extend(fixtures::left_right_frac().atoms);
    let mut c = fixtures::sum_limits();
    c.atoms.extend(fixtures::x_sub_i_sup_2().atoms);
    // x^{y^z}_{i_j}
    let y_z: MathList = Atom::symbol('y').with_sup(sym("z")).into();
    let i_j: MathList = Atom::symbol('i').with_sub(sym("j")).into();
    let nested = MathList::from(Atom::symbol('x').with_sup(y_z).with_sub(i_j));
    // \int_0^1 f(x)
    let mut int = fixtures::int_0_1();
    int.atoms.extend(fixtures::f_of_x().atoms);
    // \left[\frac{a}{b}\right]^2
    let bracket = MathList::from(
        Atom::left_right(Some('['), Some(']'), fixtures::frac_a_b()).with_sup(sym("2")),
    );
    // \hat{\imath}+\vec{x}
    let accents = MathList::new(vec![
        Atom::accent('^', sym("\u{0131}")),
        Atom::symbol('+'),
        Atom::accent('\u{20D7}', sym("x")),
    ]);
    // \sum\limits_{i=1}^{n}x_i
    let mut sum_inline = MathList::from(
        Atom::symbol('\u{2211}')
            .with_sub(sym("i=1"))
            .with_sup(sym("n"))
            .with_limits(Limits::Limits),
    );
    sum_inline.atoms.push(Atom::symbol('x').with_sub(sym("i")));
    // \prod_{k=1}^{m}a_k
    let mut prod = MathList::from(
        Atom::symbol('\u{220F}')
            .with_sub(sym("k=1"))
            .with_sup(sym("m")),
    );
    prod.atoms.push(Atom::symbol('a').with_sub(sym("k")));
    // x^2+y^2=z^2
    let pythagoras = MathList::new(vec![
        Atom::symbol('x').with_sup(sym("2")),
        Atom::symbol('+'),
        Atom::symbol('y').with_sup(sym("2")),
        Atom::symbol('='),
        Atom::symbol('z').with_sup(sym("2")),
    ]);
    // \frac{a+b}{\frac{c}{d}+e}
    let mut den: MathList = Atom::frac(sym("c"), sym("d")).into();
    den.atoms.extend(sym("+e").atoms);
    let nested_frac = MathList::from(Atom::frac(sym("a+b"), den));

    vec![
        Case {
            id: "01-stacked-fraction",
            body: strip_preamble(body!("01-stacked-fraction")),
            segments: display(a),
        },
        Case {
            id: "02-sqrt-left-right",
            body: strip_preamble(body!("02-sqrt-left-right")),
            segments: display(b),
        },
        Case {
            id: "03-sum-limits-scripts",
            body: strip_preamble(body!("03-sum-limits-scripts")),
            segments: display(c),
        },
        Case {
            id: "04-tall-braces",
            body: strip_preamble(body!("04-tall-braces")),
            segments: display(fixtures::tall_braces()),
        },
        Case {
            id: "05-cube-root",
            body: strip_preamble(body!("05-cube-root")),
            segments: display(fixtures::cube_root_frac()),
        },
        Case {
            id: "06-lim-sin",
            body: strip_preamble(body!("06-lim-sin")),
            segments: display(fixtures::lim_sin_x_over_x()),
        },
        Case {
            id: "07-tall-sqrt-braces",
            body: strip_preamble(body!("07-tall-sqrt-braces")),
            segments: display(Atom::sqrt(fixtures::tall_braces()).into()),
        },
        Case {
            id: "08-nested-scripts",
            body: strip_preamble(body!("08-nested-scripts")),
            segments: display(nested),
        },
        Case {
            id: "09-int-display",
            body: strip_preamble(body!("09-int-display")),
            segments: display(int),
        },
        Case {
            id: "10-left-bracket-frac-squared",
            body: strip_preamble(body!("10-left-bracket-frac-squared")),
            segments: display(bracket),
        },
        Case {
            id: "11-accents",
            body: strip_preamble(body!("11-accents")),
            segments: display(accents),
        },
        Case {
            id: "12-sum-limits-inline",
            body: strip_preamble(body!("12-sum-limits-inline")),
            segments: text(sum_inline),
        },
        Case {
            id: "13-bigop-scripts-inline",
            body: strip_preamble(body!("13-bigop-scripts-inline")),
            segments: text(prod),
        },
        Case {
            id: "14-mixed-text-math",
            body: strip_preamble(body!("14-mixed-text-math")),
            segments: vec![
                Segment::Word("Let".into()),
                Segment::Math(pythagoras, Style::TEXT),
                Segment::Word("hold.".into()),
            ],
        },
        Case {
            id: "15-nested-fraction-sum",
            body: strip_preamble(body!("15-nested-fraction-sum")),
            segments: display(nested_frac),
        },
    ]
}

/// The case whose declared body equals `body` byte for byte (after trimming
/// trailing whitespace on both sides).
pub fn find_by_body(body: &str) -> Option<Case> {
    let want = body.trim_end();
    cases().into_iter().find(|c| c.body.trim_end() == want)
}

/// A corpus line laid out on the page: runs in TeX points with a top-left
/// page origin, plus the line's box.
#[derive(Debug, Clone)]
pub struct LaidOutLine {
    pub runs: PositionedRuns,
    /// Left edge of the line in TeX pt from the page's left edge.
    pub x_pt: f64,
    /// Baseline in TeX pt from the page's top edge.
    pub baseline_pt: f64,
    pub width: f64,
    pub height: f64,
    pub depth: f64,
    pub limitations: Vec<Limitation>,
}

/// Lays out a case as one line at the text-area origin of a 12pt article
/// page with `margin=1in`: `x = 1in`, first baseline = `1in + max(\topskip,
/// h)` (TeX's `\topskip` rule for the first line of the page). Words are
/// roman text glyphs separated by the roman font's interword space; math
/// segments use the corpus metrics. Interword glue is at its natural width
/// (a short last line), and no text kerning/ligatures are applied.
pub fn lay_out_line(case: &Case, m: &CmMathMetrics) -> LaidOutLine {
    let mut runs = PositionedRuns::default();
    let (mut x, mut height, mut depth) = (0.0f64, 0.0f64, 0.0f64);
    let mut limitations = Vec::new();
    let space = m.text_space();
    let mut pending_space = false;
    for seg in &case.segments {
        if pending_space {
            x += space;
        }
        match seg {
            Segment::Word(w) => {
                for ch in w.chars() {
                    if let Some(g) = m.text_glyph(ch, SizeClass::Text) {
                        runs.glyphs.push(crate::boxes::PositionedGlyph {
                            font_id: g.font_id,
                            gid: g.gid,
                            ch,
                            x,
                            baseline_y: 0.0,
                            size: g.size,
                        });
                        x += g.width;
                        height = height.max(g.height);
                        depth = depth.max(g.depth);
                    } else {
                        limitations.push(Limitation::MissingGlyph(ch));
                    }
                }
            }
            Segment::Math(list, style) => {
                let l = layout_with_report(list, *style, m);
                let r = positioned_runs(&l.root, (x, -l.root.height));
                runs.glyphs.extend(r.glyphs);
                runs.rules.extend(r.rules);
                x += l.root.width;
                height = height.max(l.root.height);
                depth = depth.max(l.root.depth);
                limitations.extend(l.limitations);
            }
        }
        pending_space = true;
    }
    LaidOutLine {
        runs,
        x_pt: MARGIN_PT,
        baseline_pt: MARGIN_PT + TOPSKIP_PT.max(height),
        width: x,
        height,
        depth,
        limitations,
    }
}
