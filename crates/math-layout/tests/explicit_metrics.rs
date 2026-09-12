//! Geometry tests against a synthetic, fully explicit metrics provider.
//!
//! Unlike `tests/golden.rs` (which cross-checks the Computer Modern adapter
//! against pdfTeX `\showbox` output), every number here is a literal chosen
//! by this file: glyph widths/heights/depths, radical and delimiter size
//! chains, and every `MathParams` field the exercised code paths read. The
//! expected geometry in each assertion is derived by hand from those same
//! literals against the TeXbook Appendix G rule cited in the comment, so the
//! test does not depend on any value the layout engine itself computed (no
//! TFM parsing, no font adapter). FT-032 revision 3, part 2/3.

use flashtex_math_layout::metrics::Extensible;
use flashtex_math_layout::{
    Atom, FontId, Glyph, MathFontMetrics, MathList, MathParams, SizeClass, Style, layout,
    layout_with_report, positioned_runs,
};

fn f5(v: f64) -> String {
    format!("{:.5}", v)
}

fn dims(b: &flashtex_math_layout::MathBox) -> String {
    format!("({}+{})x{}", f5(b.height), f5(b.depth), f5(b.width))
}

/// One glyph literal: `font_id` is always `FontId(1)` (there is only one
/// synthetic font here) and `gid` is the character's own code point,
/// truncated to 16 bits (every character used below is in the BMP, so this
/// is exact). `skew` is always 0 — no accent test uses this provider.
fn g(ch: char, size: f64, width: f64, height: f64, depth: f64, italic: f64) -> Glyph {
    Glyph {
        font_id: FontId(1),
        gid: ch as u32 as u16,
        ch,
        size,
        width,
        height,
        depth,
        italic,
        skew: 0.0,
    }
}

/// Every `MathParams` field this file's tests actually read, spelled out
/// literally per size class. Fields no exercised code path reads (the
/// `big_op_spacing*` family, `delim1`/`delim2`, `num*`/`denom*`, the unused
/// half of `sup*`/`sub*`/`sup_drop`/`sub_drop`) are still given explicit,
/// finite numbers so the struct is fully inhabited, but no assertion below
/// depends on their values.
struct Metrics {
    text: MathParams,
    script: MathParams,
    scriptscript: MathParams,
}

impl Metrics {
    fn new() -> Metrics {
        // Text size (also what `Style::DISPLAY` and `Style::TEXT` both read,
        // since `size_class()` maps both to `SizeClass::Text`): x_height =
        // 4.0, default_rule_thickness (theta) = 0.4, quad = 18.0 so
        // `MathParams::mu()` truncates to exactly 1.0 (round(18*65536) =
        // 1_179_648, trunc(1_179_648 / 18) = 65_536), axis_height = 2.5,
        // sup2 = 3.0, sub2 = 2.5, delimiter_factor = 0.9, delimiter_shortfall
        // = 5.0, script_space = 0.5, null_delimiter_space = 1.2.
        let text = MathParams {
            size: 10.0,
            x_height: 4.0,
            quad: 18.0,
            num1: 6.0,
            num2: 4.0,
            num3: 4.0,
            denom1: 6.0,
            denom2: 4.0,
            sup1: 4.0,
            sup2: 3.0,
            sup3: 2.5,
            sub1: 3.0,
            sub2: 2.5,
            sup_drop: 1.0,
            sub_drop: 1.0,
            delim1: 20.0,
            delim2: 15.0,
            axis_height: 2.5,
            default_rule_thickness: 0.4,
            big_op_spacing1: 1.0,
            big_op_spacing2: 1.0,
            big_op_spacing3: 1.0,
            big_op_spacing4: 1.0,
            big_op_spacing5: 1.0,
            script_space: 0.5,
            null_delimiter_space: 1.2,
            delimiter_factor: 0.9,
            delimiter_shortfall: 5.0,
        };
        // Script size: only `sup_drop` (2.0) and `sub_drop` (0.0) are read
        // (as `t.sup_drop`/`t.sub_drop` in Rule 18's shift seeding, from the
        // *superscript* style's params); the rest is filler.
        let script = MathParams {
            size: 7.0,
            x_height: 2.8,
            quad: 12.6,
            num1: 4.2,
            num2: 2.8,
            num3: 2.8,
            denom1: 4.2,
            denom2: 2.8,
            sup1: 2.8,
            sup2: 2.1,
            sup3: 1.75,
            sub1: 2.1,
            sub2: 1.75,
            sup_drop: 2.0,
            sub_drop: 0.0,
            delim1: 14.0,
            delim2: 10.5,
            axis_height: 1.75,
            default_rule_thickness: 0.28,
            big_op_spacing1: 0.7,
            big_op_spacing2: 0.7,
            big_op_spacing3: 0.7,
            big_op_spacing4: 0.7,
            big_op_spacing5: 0.7,
            script_space: 0.35,
            null_delimiter_space: 0.84,
            delimiter_factor: 0.9,
            delimiter_shortfall: 3.5,
        };
        // Scriptscript size: read only through `MathParams::mu()` (for the
        // `\r@@t` mkerns) when a degree is itself laid out recursively;
        // none of the tests below nest that deep, so this is pure filler.
        let scriptscript = MathParams {
            size: 5.0,
            x_height: 2.0,
            quad: 9.0,
            num1: 3.0,
            num2: 2.0,
            num3: 2.0,
            denom1: 3.0,
            denom2: 2.0,
            sup1: 2.0,
            sup2: 1.5,
            sup3: 1.25,
            sub1: 1.5,
            sub2: 1.25,
            sup_drop: 1.0,
            sub_drop: 0.0,
            delim1: 10.0,
            delim2: 7.5,
            axis_height: 1.25,
            default_rule_thickness: 0.2,
            big_op_spacing1: 0.5,
            big_op_spacing2: 0.5,
            big_op_spacing3: 0.5,
            big_op_spacing4: 0.5,
            big_op_spacing5: 0.5,
            script_space: 0.25,
            null_delimiter_space: 0.6,
            delimiter_factor: 0.9,
            delimiter_shortfall: 2.5,
        };
        Metrics {
            text,
            script,
            scriptscript,
        }
    }
}

impl MathFontMetrics for Metrics {
    fn params(&self, size: SizeClass) -> MathParams {
        match size {
            SizeClass::Text => self.text,
            SizeClass::Script => self.script,
            SizeClass::ScriptScript => self.scriptscript,
        }
    }

    fn font_name(&self, _font: FontId) -> String {
        "explicit-test".to_string()
    }

    fn glyph(&self, ch: char, size: SizeClass) -> Option<Glyph> {
        use SizeClass::{Script, ScriptScript, Text};
        Some(match (ch, size) {
            // Radicand of the indexed-root test (part 2).
            ('x', Text) => g('x', 10.0, 5.0, 4.0, 1.0, 0.0),
            // Degree of the indexed-root test: `\sqrt[n]{}` always sets the
            // degree in scriptscript style (`make_radical`), regardless of
            // the outer style.
            ('n', ScriptScript) => g('n', 5.0, 2.0, 1.5, 0.0, 0.0),
            // Tall delimited body (part 3, delimiter sizing).
            ('X', Text) => g('X', 10.0, 5.0, 9.0, 1.0, 0.0),
            // Non-bare-character nucleus for the both-scripts test (part 3).
            ('A', Text) => g('A', 10.0, 3.0, 2.0, 0.5, 0.0),
            // Superscript and subscript glyphs, laid out in script style.
            ('p', Script) => g('p', 7.0, 2.0, 1.0, 3.0, 0.0),
            ('q', Script) => g('q', 7.0, 2.0, 3.0, 0.5, 0.0),
            _ => return None,
        })
    }

    fn large_operator(&self, _ch: char, _size: SizeClass) -> Option<Glyph> {
        None
    }

    fn delimiter_sizes(&self, ch: char, _size: SizeClass) -> Vec<Glyph> {
        // Two-size chain, smallest first, identical at every size class
        // (only `SizeClass::Text` is exercised by an assertion below): a
        // small variant that Rule 19's `wanted` will reject, and a larger
        // one it accepts. `(` and `)` are given different widths so the
        // test can tell the two chosen glyphs apart without relying on
        // symmetry.
        match ch {
            '(' => vec![
                g('(', 10.0, 3.0, 5.0, 5.0, 0.0),
                g('(', 10.0, 4.0, 7.0, 7.0, 0.0),
            ],
            ')' => vec![
                g(')', 10.0, 3.0, 5.0, 5.0, 0.0),
                g(')', 10.0, 4.5, 7.0, 7.0, 0.0),
            ],
            _ => Vec::new(),
        }
    }

    fn radical_sizes(&self, _size: SizeClass) -> Vec<Glyph> {
        // A single radical-sign size, identical at every size class: total
        // height 9.5 (0.5 + 9.0), comfortably above every `wanted` value
        // this file computes, so `var_delimiter` always picks it directly
        // and no extensible recipe is needed.
        vec![g('\u{221A}', 10.0, 1.0, 0.5, 9.0, 0.0)]
    }

    fn accent_sizes(&self, _ch: char, _size: SizeClass) -> Vec<Glyph> {
        Vec::new()
    }

    fn delimiter_extensible(&self, _ch: char, _size: SizeClass) -> Option<Extensible> {
        None
    }

    fn radical_extensible(&self, _size: SizeClass) -> Option<Extensible> {
        None
    }
}

/// `\sqrt[n]{x}` in display style (Rule 11 plus LaTeX's `\r@@t` degree
/// placement), against the explicit metrics above.
///
/// Radicand: `x` is (4.0+1.0)x5.0. Style is display and uncramped, but the
/// radicand is boxed cramped (`make_sqrt`); cramping does not change this
/// glyph's own box, so `x` (the box) is (4.0+1.0)x5.0 too.
///
/// clr = theta + x_height/4 = 0.4 + 1.0 = 1.4 (display branch of Rule 11).
/// wanted = h(x)+d(x)+clr+theta = 4.0+1.0+1.4+0.4 = 6.8. The only radical
/// sign (total height 9.5) clears that, so it is chosen directly.
///
/// delta = d(sign) - (h(x)+d(x)+clr) = 9.0 - 6.4 = 2.6 > 0, so
/// clr += delta/2 = 1.4 + 1.3 = 2.7.
///
/// sign_dy = -(h(x)+clr) = -6.7. rule_thickness = h(sign) = 0.5.
/// overbar height = h(x)+clr+2*rule_thickness = 4.0+2.7+1.0 = 7.7,
/// depth = d(x) = 1.0.
///
/// z = hbox[(sign_dy=-6.7, sign), (0, overbar)]: height = max(h(sign)-dy,
/// h(overbar)) = max(0.5+6.7, 7.7) = 7.7; depth = max(d(sign)+dy,
/// d(overbar)) = max(9.0-6.7, 1.0) = 2.3; width = w(sign) + w(overbar) =
/// 1.0 + 5.0 = 6.0.
///
/// Degree: mu = 1.0 (quad=18.0 truncates exactly). raise = 0.6*(h(z)-d(z))
/// = 0.6*(7.7-2.3) = 3.24. `n` (scriptscript) is (1.5+0.0)x2.0.
///
/// Final hbox: kern(5mu=5.0), n shifted -3.24, kern(-10mu=-10.0), z.
/// height = max(0, 1.5+3.24, 0, 7.7) = 7.7. depth = max(0, 0.0-3.24, 0, 2.3)
/// = 2.3 (the -3.24 term is negative and does not raise the max). width =
/// 5.0 + 2.0 - 10.0 + 6.0 = 3.0.
#[test]
fn indexed_root_degree_and_rule_geometry_from_explicit_metrics() {
    let m = Metrics::new();
    let list = MathList::from(Atom::root(
        MathList::from(Atom::ord('n')),
        MathList::from(Atom::ord('x')),
    ));
    let b = layout(&list, Style::DISPLAY, &m);
    assert_eq!(dims(&b), "(7.70000+2.30000)x3.00000");

    let r = positioned_runs(
        &layout_with_report(&list, Style::DISPLAY, &m).root,
        (0.0, 0.0),
    );
    let find = |ch: char| r.glyphs.iter().find(|glyph| glyph.ch == ch).unwrap();
    let n = find('n');
    let sign = find('\u{221A}');
    let x = find('x');

    // The degree's baseline is raised by exactly 3.24pt above the box
    // baseline (7.7), and starts right after the 5mu kern.
    assert_eq!(f5(b.height - n.baseline_y), "3.24000");
    assert_eq!(f5(n.x), "5.00000");

    // The sign's baseline is 6.7pt above the box baseline (h(x)+clr).
    assert_eq!(f5(b.height - sign.baseline_y), "6.70000");
    assert_eq!(f5(sign.x), "-3.00000");

    // The radicand sits exactly on the box baseline (nothing shifts it) at
    // x = -2.0 (z's dx(-3.0) + the sign's width(1.0)).
    assert_eq!(f5(x.baseline_y), f5(b.height));
    assert_eq!(f5(x.x), "-2.00000");

    // The overbar rule: thickness = h(sign) = 0.5, same width as x (5.0),
    // same x as x. The bar box has zero depth, so its *baseline* sits
    // h(x)+clr = 6.7pt above the box baseline (at 7.7-6.7 = 1.0 from the
    // top edge); the rule's own top edge (`PositionedRule::y`) is a further
    // rule-thickness above that: 1.0 - 0.5 = 0.5.
    assert_eq!(r.rules.len(), 1);
    assert_eq!(f5(r.rules[0].h), "0.50000");
    assert_eq!(f5(r.rules[0].w), "5.00000");
    assert_eq!(f5(r.rules[0].x), f5(x.x));
    assert_eq!(f5(r.rules[0].y), "0.50000");

    assert!(
        layout_with_report(&list, Style::DISPLAY, &m)
            .limitations
            .is_empty()
    );
}

/// `\left( X \right)` in text style (Rule 19), against the explicit
/// metrics above: `X` is a single, deliberately tall/deep glyph so the
/// wanted delimiter height is computed from a literal, not a nested
/// construct.
///
/// inner = X = (9.0+1.0)x5.0. delta1 = max(h-a, d+a) = max(9.0-2.5,
/// 1.0+2.5) = max(6.5, 3.5) = 6.5. wanted = max(2*delta1*factor,
/// 2*delta1-shortfall) = max(2*6.5*0.9, 2*6.5-5.0) = max(11.7, 8.0) = 11.7.
///
/// `(`'s small size (total height 10.0) is below 11.7 and rejected; the
/// large size (total height 14.0) clears it and is chosen; same for `)`.
/// Both chosen glyphs have height=depth=7.0, so
/// shift = (h-d)/2 - axis_height = 0 - 2.5 = -2.5 for both.
///
/// Final box: open (h=7.0-(-2.5)=9.5, d=7.0+(-2.5)=4.5, w=4.0), inner
/// (h=9.0, d=1.0, w=5.0), close (h=9.5, d=4.5, w=4.5, from the *wider*
/// large `)`). height = max(9.5,9.0,9.5) = 9.5, depth = max(4.5,1.0,4.5) =
/// 4.5, width = 4.0+5.0+4.5 = 13.5.
#[test]
fn delimiter_sizing_against_explicit_metrics() {
    let m = Metrics::new();
    let list = MathList::from(Atom::left_right(
        Some('('),
        Some(')'),
        MathList::from(Atom::ord('X')),
    ));
    let out = layout_with_report(&list, Style::TEXT, &m);
    assert_eq!(dims(&out.root), "(9.50000+4.50000)x13.50000");
    assert!(out.limitations.is_empty());

    let r = positioned_runs(&out.root, (0.0, 0.0));
    let open = r.glyphs.iter().find(|g| g.ch == '(').unwrap();
    let close = r.glyphs.iter().find(|g| g.ch == ')').unwrap();
    let body = r.glyphs.iter().find(|g| g.ch == 'X').unwrap();

    // Both delimiters picked the *large* size (width 4.0 / 4.5, not the
    // small chain's 3.0), confirming Rule 19 rejected the small variant.
    assert_eq!(f5(close.x - open.x), f5(4.0 + 5.0));
    assert_eq!(f5(out.root.width - close.x), "4.50000");

    // Both are centred on the axis: the formula baseline (9.5 from the top)
    // minus the delimiter baseline is 2.5 (== axis_height, since h=d=7.0
    // makes the un-axis-shifted centre coincide with the delimiter's own
    // baseline).
    assert_eq!(f5(9.5 - open.baseline_y), "2.50000");
    assert_eq!(f5(open.baseline_y), f5(close.baseline_y));

    // The body glyph is untouched by the delimiter sizing: its own baseline
    // is the formula baseline.
    assert_eq!(f5(body.baseline_y), "9.50000");
}

/// Superscript and subscript together (Rule 18d/e) on a **non-bare-character
/// nucleus** (`{A}`, a `Nucleus::List`, so `is_char` is false and the shift
/// seeds come from the nucleus box's own height/depth minus
/// `sup_drop`/`sub_drop` — the branch `nested_scripts_x_sub_i_sup_2` in
/// `golden.rs` does not cover, since its nucleus is a bare `x`), chosen so
/// both stages of the Rule 18e clearance correction fire.
///
/// Nucleus `{A}` = (2.0+0.5)x3.0 (a one-atom group, so its box is exactly
/// the glyph's). shift_up seed = h(nucleus) - sup_drop(script) = 2.0 - 2.0
/// = 0.0. shift_down seed = d(nucleus) + sub_drop(script) = 0.5 + 0.0 =
/// 0.5.
///
/// Superscript `p` = (1.0+3.0)x2.0. clr = sup2(text) = 3.0 →
/// shift_up = max(0.0, 3.0) = 3.0. clr2 = d(p) + x_height/4 = 3.0 + 1.0 =
/// 4.0 → shift_up = max(3.0, 4.0) = 4.0.
///
/// Subscript `q` = (3.0+0.5)x2.0. shift_down = max(0.5, sub2(text)=2.5) =
/// 2.5.
///
/// Rule 18e gap check: clr = 4*theta - ((shift_up - d(p)) - (h(q) -
/// shift_down)) = 1.6 - ((4.0-3.0) - (3.0-2.5)) = 1.6 - 0.5 = 1.1 > 0, so
/// shift_down += 1.1 → 3.6. Second check: clr2 = x_height*4/5 -
/// (shift_up - d(p)) = 3.2 - 1.0 = 2.2 > 0, so shift_up += 2.2 → 6.2 and
/// shift_down -= 2.2 → 1.4.
///
/// Final: width = max(w(p)+script_space, w(q)+script_space) = max(2.5,2.5)
/// = 2.5 (delta = 0, since the nucleus is not a bare character). height =
/// h(p) + shift_up = 1.0+6.2 = 7.2. depth = d(q) + shift_down = 0.5+1.4 =
/// 1.9. Outer box: height = max(2.0, 7.2) = 7.2, depth = max(0.5, 1.9) =
/// 1.9, width = w(nucleus) + 2.5 = 3.0 + 2.5 = 5.5.
#[test]
fn both_scripts_on_non_bare_nucleus_exercise_the_rule_18e_clearance() {
    let m = Metrics::new();
    let nucleus = Atom::group(MathList::from(Atom::ord('A')));
    let list = MathList::from(
        nucleus
            .with_sup(MathList::from(Atom::ord('p')))
            .with_sub(MathList::from(Atom::ord('q'))),
    );
    let b = layout(&list, Style::TEXT, &m);
    assert_eq!(dims(&b), "(7.20000+1.90000)x5.50000");

    let r = positioned_runs(&layout_with_report(&list, Style::TEXT, &m).root, (0.0, 0.0));
    let a = r.glyphs.iter().find(|g| g.ch == 'A').unwrap();
    let p = r.glyphs.iter().find(|g| g.ch == 'p').unwrap();
    let q = r.glyphs.iter().find(|g| g.ch == 'q').unwrap();

    // The nucleus sits on the formula baseline; both scripts start right
    // after it, at the same x (delta = 0 for a non-char nucleus).
    assert_eq!(f5(a.baseline_y), f5(b.height));
    assert_eq!(f5(a.x), "0.00000");
    assert_eq!(f5(p.x), "3.00000");
    assert_eq!(f5(q.x), "3.00000");

    // shift_up = 6.2 after both Rule 18e corrections; shift_down = 1.4.
    assert_eq!(f5(b.height - p.baseline_y), "6.20000");
    assert_eq!(f5(q.baseline_y - b.height), "1.40000");
}

/// Building the same formula twice, from two independently-constructed
/// `MathList` values (never `.clone()`d from one another) and two
/// independently-constructed metrics providers, against every style, must
/// give byte-identical geometry. `flashtex_math_layout` holds no cache, no
/// hash map, and no other memoized/global state (`grep -rn "HashMap\|Cache"
/// crates/math-layout/src` is empty) — this pins that down as an explicit,
/// permanent contract rather than an accident of the current
/// implementation, and checks the f64 bit patterns directly (not just
/// `PartialEq`, which cannot tell `0.0` from `-0.0` and treats two `NaN`s
/// as unequal) so a future change that introduces even a sign-of-zero or
/// summation-order dependency would fail here.
#[test]
fn clean_rebuild_from_scratch_is_byte_identical() {
    fn build() -> MathList {
        let root = Atom::root(
            MathList::from(Atom::ord('n')),
            MathList::from(Atom::ord('x')),
        )
        .with_sup(MathList::from(Atom::ord('p')))
        .with_sub(MathList::from(Atom::ord('q')));
        let parenthesized = Atom::left_right(Some('('), Some(')'), MathList::from(Atom::ord('X')));
        MathList::new(vec![root, parenthesized])
    }

    let list_a = build();
    let list_b = build();
    assert_eq!(list_a, list_b);
    // These are two distinct `Vec` allocations, not the same value reused.
    assert_ne!(list_a.atoms.as_ptr(), list_b.atoms.as_ptr());

    for style in [
        Style::DISPLAY,
        Style::TEXT,
        Style::SCRIPT,
        Style::SCRIPT_SCRIPT,
    ] {
        let out_a = layout_with_report(&list_a, style, &Metrics::new());
        let out_b = layout_with_report(&list_b, style, &Metrics::new());
        assert_eq!(out_a, out_b, "geometry differs for style {style:?}");
        assert_eq!(out_a.root.height.to_bits(), out_b.root.height.to_bits());
        assert_eq!(out_a.root.depth.to_bits(), out_b.root.depth.to_bits());
        assert_eq!(out_a.root.width.to_bits(), out_b.root.width.to_bits());

        let runs_a = positioned_runs(&out_a.root, (72.0, 700.0));
        let runs_b = positioned_runs(&out_b.root, (72.0, 700.0));
        assert_eq!(runs_a, runs_b);
        for (ga, gb) in runs_a.glyphs.iter().zip(&runs_b.glyphs) {
            assert_eq!(ga.x.to_bits(), gb.x.to_bits());
            assert_eq!(ga.baseline_y.to_bits(), gb.baseline_y.to_bits());
        }
        for (ra, rb) in runs_a.rules.iter().zip(&runs_b.rules) {
            assert_eq!(ra.x.to_bits(), rb.x.to_bits());
            assert_eq!(ra.y.to_bits(), rb.y.to_bits());
            assert_eq!(ra.w.to_bits(), rb.w.to_bits());
            assert_eq!(ra.h.to_bits(), rb.h.to_bits());
        }
    }

    // Rebuilding from scratch repeatedly (fresh list, fresh provider, every
    // time) never drifts from the first result.
    let baseline = layout_with_report(&build(), Style::TEXT, &Metrics::new());
    for _ in 0..25 {
        let out = layout_with_report(&build(), Style::TEXT, &Metrics::new());
        assert_eq!(out, baseline);
    }
}
