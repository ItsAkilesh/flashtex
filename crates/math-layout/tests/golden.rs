//! Golden geometry tests against the Computer Modern metrics.
//!
//! Every expected number below was cross-checked against pdfTeX 1.40.29
//! (TeX Live 2026, BasicTeX) `\showbox` output for the same LaTeX source at
//! 10pt; the derivations name the Appendix G rule and the parameters that
//! produce the value. Numbers are compared at TeX's own `\showbox` precision
//! (five decimals) unless stated otherwise. The oracle is used only as
//! evidence; nothing here runs TeX.

use flashtex_math_layout::{
    Atom, BoxKind, CmMathMetrics, Limitation, MathBox, MathFontMetrics, MathList, PositionedRuns,
    Style, TimesApproxMetrics, fixtures, layout, layout_with_report, positioned_runs,
};

fn cm() -> CmMathMetrics {
    CmMathMetrics::latex_10pt()
}

fn f5(v: f64) -> String {
    format!("{:.5}", v)
}

/// `(height+depth)xwidth` at five decimals, like `\showbox`.
fn dims(b: &MathBox) -> String {
    format!("({}+{})x{}", f5(b.height), f5(b.depth), f5(b.width))
}

fn runs(list: &MathList, style: Style) -> PositionedRuns {
    positioned_runs(&layout(list, style, &cm()), (0.0, 0.0))
}

fn glyph(r: &PositionedRuns, ch: char) -> &flashtex_math_layout::PositionedGlyph {
    r.glyphs.iter().find(|g| g.ch == ch).unwrap()
}

#[test]
fn superscript_x_squared() {
    // Rule 18c: a bare character nucleus starts with shift_up = 0, which is
    // raised to sup2 = 3.62892pt (cmsy10 fontdimen 14 = 0.362892 em) because
    // the text style is not cramped and not display. The clearance test
    // d(sup) + x_height/4 = 1.07639 is smaller and does not apply.
    // \showbox: \hbox(8.14003+0.0)x10.2014, superscript "shifted -3.62892".
    let b = layout(&fixtures::x_squared(), Style::TEXT, &cm());
    assert_eq!(dims(&b), "(8.14003+0.00000)x10.20140");
    let r = runs(&fixtures::x_squared(), Style::TEXT);
    let x = glyph(&r, 'x');
    let two = glyph(&r, '2');
    assert_eq!(f5(x.baseline_y - two.baseline_y), "3.62892");
    // The superscript starts right after x's advance (5.71527pt, cmmi10) and
    // its box is 3.98613 + 0.5pt \scriptspace wide.
    assert_eq!(f5(two.x), "5.71527");
    assert_eq!(f5(b.width - two.x), "4.48613");
    assert_eq!(two.size, 7.0);
}

#[test]
fn nested_scripts_x_sub_i_sup_2() {
    // Rule 18e: shift_down starts at sub2 = 2.47217pt; the gap between the
    // superscript bottom and subscript top must be at least 4θ = 1.59991pt
    // (θ = 0.39998pt, cmex10 fontdimen 8), so shift_down grows by 0.13075pt
    // to 2.60292pt. The x-height*4/5 test (3.44444 - 3.62892 < 0) is inactive.
    // \showbox: \hbox(8.14003+2.60292)x10.2014, vbox shifted 2.60292,
    // \kern1.59991 between the scripts.
    let list = fixtures::x_sub_i_sup_2();
    let b = layout(&list, Style::TEXT, &cm());
    assert_eq!(dims(&b), "(8.14003+2.60292)x10.20140");
    let r = runs(&list, Style::TEXT);
    let base = glyph(&r, 'x').baseline_y;
    assert_eq!(f5(glyph(&r, 'i').baseline_y - base), "2.60292");
    assert_eq!(f5(base - glyph(&r, '2').baseline_y), "3.62892");
    // The sub and sup start at the same x (δ = 0 for x, whose italic
    // correction is zero).
    assert_eq!(f5(glyph(&r, 'i').x), f5(glyph(&r, '2').x));
}

#[test]
fn nested_scripts_x_sup_y_sup_z() {
    // The inner y^z is set in script style: its superscript shift is sup2 of
    // cmsy7 = 0.431115 em * 7pt = 3.01779pt, and z is a 5pt (scriptscript)
    // glyph. y's italic correction (0.25116pt at 7pt) is appended as a kern
    // because y has no subscript (Rule 17), so z starts after it.
    // \showbox: \hbox(8.79948+0.0)x14.6505; inner hbox(5.17056+1.3611)x8.93523
    // shifted -3.62892; \kern0.25116; z box shifted -3.01779.
    let list = fixtures::x_sup_y_sup_z();
    let b = layout(&list, Style::TEXT, &cm());
    assert_eq!(dims(&b), "(8.79948+0.00000)x14.65050");
    let r = runs(&list, Style::TEXT);
    let (x, y, z) = (glyph(&r, 'x'), glyph(&r, 'y'), glyph(&r, 'z'));
    assert_eq!(f5(x.baseline_y - y.baseline_y), "3.62892");
    assert_eq!(f5(y.baseline_y - z.baseline_y), "3.01779");
    assert_eq!((y.size, z.size), (7.0, 5.0));
    assert_eq!(f5(z.x - y.x), f5(4.05559 + 0.25116));
}

#[test]
fn stacked_fraction_text_style() {
    // Rule 15, text style (θ ≠ 0): u = num2 = 3.93732, v = denom2 = 3.44841.
    // The numerator is itself a fraction in script style with u' = num2(7pt)
    // = 2.68732 and v' = 2.4095 (no clearance adjustment needed), giving a
    // numerator box (4.84009+2.4095). The outer numerator clearance
    // δ1 = θ - ((u - d(x)) - (a + θ/2)) = 0.39998 - (1.52782 - 2.69999)
    // = 1.57215 > 0 raises u to 5.50947, so height = 4.84009 + 5.50947.
    // Family 3 stays at 10pt in LaTeX (omxcmex.fd: sfixed*cmex10), so the
    // inner rule is also 0.39998pt thick, not 0.28pt.
    // \showbox: \hbox(10.34956+3.44841)x8.67213.
    let list = fixtures::stacked_fraction();
    let b = layout(&list, Style::TEXT, &cm());
    assert_eq!(dims(&b), "(10.34956+3.44841)x8.67213");
    let r = runs(&list, Style::TEXT);
    assert_eq!(r.rules.len(), 2);
    let inner = &r.rules[0];
    let outer = &r.rules[1];
    assert_eq!(f5(inner.h), "0.39998");
    assert_eq!(f5(outer.h), "0.39998");
    // The outer rule is centred on the text axis (2.5pt above the baseline):
    // its top edge is at height - (a + θ/2) from the top of the box.
    assert_eq!(f5(outer.y), f5(10.34956 - 2.5 - 0.39998 / 2.0));
    // The inner rule is centred on the script axis (1.75pt) relative to the
    // numerator baseline, which sits u = 5.50947pt above the outer baseline.
    let num_baseline = 10.34956 - 5.50947;
    assert_eq!(f5(inner.y), f5(num_baseline - 1.75 - 0.39998 / 2.0));
    // Null delimiters add 1.2pt (78643sp) on each side of both fractions.
    assert_eq!(f5(outer.x), "1.20000");
    assert_eq!(f5(inner.x), "2.39999");
    assert_eq!(f5(outer.w), "6.27214");
    // Sizes: a, b are scriptscript (5pt), c is script (7pt).
    assert_eq!(glyph(&r, 'a').size, 5.0);
    assert_eq!(glyph(&r, 'c').size, 7.0);
}

#[test]
fn stacked_fraction_display_style() {
    // Display: u = num1 = 6.76508, v = denom1 = 6.85951, clearance 3θ.
    // The numerator (a text-style fraction) is (6.9512+3.44841) tall, so
    // δ1 = 3θ - ((6.76508 - 3.44841) - 2.69999) = 1.19994 - 0.61668 = 0.58326
    // lifts u to 7.34834; height = 6.9512 + 7.34834 = 14.29954.
    // \showbox: \hbox(14.29955+6.85951)x9.13763 (1sp rounding in TeX).
    let b = layout(&fixtures::stacked_fraction(), Style::DISPLAY, &cm());
    assert_eq!(dims(&b), "(14.29954+6.85951)x9.13763");
}

#[test]
fn radical_overbar_geometry() {
    // Rule 11, text style: ψ = θ + θ/4 = 0.49998; the sign wanted is
    // h+d+ψ+θ = 5.2055pt, so the small cmsy10 sign (0.39998+9.60001) is
    // chosen. Its depth exceeds h(x)+d(x)+ψ by 4.79448, half of which is
    // added to ψ → 2.89722. The sign is raised by h(x)+ψ = 7.20276 so its
    // top meets the rule; overbar = kern θ' + rule θ' + kern ψ + x, with
    // θ' = h(sign) = 0.39998, giving height 4.30554 + 2.89722 + 2*0.39998.
    // \showbox: \hbox(8.00272+2.39725)x14.04863, sign shifted -7.20276,
    // \kern2.89722 under the rule.
    let list = fixtures::sqrt_x();
    let b = layout(&list, Style::TEXT, &cm());
    assert_eq!(dims(&b), "(8.00272+2.39725)x14.04863");
    let r = runs(&list, Style::TEXT);
    let sign = glyph(&r, '\u{221A}');
    let x = glyph(&r, 'x');
    assert_eq!(f5(x.baseline_y - sign.baseline_y), "7.20276");
    let rule = &r.rules[0];
    assert_eq!(f5(rule.h), "0.39998");
    assert_eq!(f5(rule.x), "8.33336");
    assert_eq!(f5(rule.w), "5.71527");
    // Top of the rule = top of the sign = one θ' below the box top.
    assert_eq!(f5(rule.y), "0.39998");
    assert_eq!(f5(sign.baseline_y - 0.39998), f5(rule.y));
    // Gap between the rule bottom and the top of x is ψ = 2.89722.
    assert_eq!(f5((x.baseline_y - 4.30554) - (rule.y + rule.h)), "2.89722");
}

#[test]
fn radical_picks_a_larger_sign_for_a_fraction() {
    // The radicand (6.9512+3.44841) wants 11.29957pt; cmsy10's sign (10pt)
    // is too small so the first cmex10 variant (0.39998+11.60013) is used.
    // \showbox: \hbox(8.60141+3.79868)x16.73766, \OMX/cmex ^^p shifted -7.80145.
    let list = fixtures::sqrt_frac();
    let out = layout_with_report(&list, Style::TEXT, &cm());
    assert_eq!(dims(&out.root), "(8.60140+3.79868)x16.73766");
    assert!(out.limitations.is_empty());
    let r = runs(&list, Style::TEXT);
    let sign = glyph(&r, '\u{221A}');
    assert_eq!(sign.gid, 0x70);
    assert_eq!(cm().font_name(sign.font_id), "cmex10");
}

#[test]
fn delimiter_sizing_left_right_fraction() {
    // Rule 19: the body is (6.9512+3.44841); δ = max(h - a, d + a) = 5.94841;
    // wanted = max(2δ·901/1000, 2δ - 5pt) = 10.719pt. cmr10 "(" is 10pt tall,
    // so the first cmex10 variant (12pt: 0.39998+11.60013) is chosen and
    // centred on the axis: shift = (h - d)/2 - a = -8.10007.
    // \showbox: \hbox(8.50005+3.50006)x15.90436, ^^@ shifted -8.10007.
    let list = fixtures::left_right_frac();
    let out = layout_with_report(&list, Style::TEXT, &cm());
    assert_eq!(dims(&out.root), "(8.50005+3.50005)x15.90436");
    assert!(out.limitations.is_empty());
    let r = runs(&list, Style::TEXT);
    let open = glyph(&r, '(');
    let close = glyph(&r, ')');
    assert_eq!((open.gid, close.gid), (0x00, 0x01));
    assert_eq!(cm().font_name(open.font_id), "cmex10");
    // Baseline of the delimiter is 8.10007 above the formula baseline.
    let base = 8.50005;
    assert_eq!(f5(base - open.baseline_y), "8.10007");
    assert_eq!(f5(open.baseline_y), f5(close.baseline_y));
}

#[test]
fn delimiter_falls_back_to_largest_and_reports() {
    // A 60pt-tall body exceeds the largest non-extensible cmex10 parenthesis
    // (30pt). The engine uses the 30pt glyph and reports the shortfall
    // instead of silently drawing something else.
    let tall = Atom::sqrt(MathList::from(Atom::sqrt(MathList::from(Atom::sqrt(
        MathList::from(Atom::frac(
            fixtures::stacked_fraction(),
            fixtures::stacked_fraction(),
        )),
    )))));
    let list = MathList::from(Atom::left_right(Some('('), Some(')'), tall.into()));
    let out = layout_with_report(&list, Style::DISPLAY, &cm());
    let hit = out.limitations.iter().find_map(|l| match l {
        Limitation::DelimiterTooSmall { ch, wanted, used } => Some((*ch, *wanted, *used)),
        _ => None,
    });
    let (ch, wanted, used) = hit.expect("shortfall reported");
    assert_eq!(ch, '(');
    assert!(wanted > used);
    assert_eq!(f5(used), "30.00029");
}

#[test]
fn sum_with_limits_in_display_style() {
    // Rule 13a: in display style \sum takes the large cmex10 variant (gid
    // 0x58, (1.0+15.00012)x14.44447) centred on the axis: shift = (1-15)/2 -
    // 2.5 = -9.50006, so the nucleus box is (10.50006+5.50006). The upper
    // limit sits max(ξ11 - d(x), ξ9) = max(1.99998, 1.1111) = 1.99998 above
    // it plus ξ13 = 1.0 on top; the lower limit max(ξ12 - h(z), ξ10) =
    // max(6.0 - 4.63193, 1.66666) = 1.66666 below plus 1.0.
    // \showbox: \hbox(16.51393+12.79865)x14.44447, kerns 1.0/1.99998/1.66666/1.0.
    let list = fixtures::sum_limits();
    let b = layout(&list, Style::DISPLAY, &cm());
    assert_eq!(dims(&b), "(16.51393+12.79865)x14.44447");
    let r = runs(&list, Style::DISPLAY);
    let sum = glyph(&r, '\u{2211}');
    assert_eq!(sum.gid, 0x58);
    let n = glyph(&r, 'n');
    let i = glyph(&r, 'i');
    // Limits are centred over the operator's 14.44447pt width.
    assert_eq!(f5(n.x), f5((14.44447 - 4.94333) / 2.0));
    assert_eq!(f5(i.x), f5((14.44447 - 12.95433) / 2.0));
    // Baselines: n at 1.0 + 3.01389 from the top; the sum's baseline 9.50006
    // above the formula baseline (16.51393 from the top).
    assert_eq!(f5(n.baseline_y), "4.01389");
    assert_eq!(f5(sum.baseline_y), f5(16.51393 - 9.50006));
    assert_eq!(f5(i.baseline_y), f5(16.51393 + 12.79865 - 1.0));
}

#[test]
fn sum_with_scripts_in_text_style() {
    // Rule 13 (no limits): the text-size \sum (gid 0x50, (0+10.00012)) is
    // centred on the axis (shift -7.50006) and, being a box rather than a
    // character, seeds shift_up = h - sup_drop(7pt) = 7.50006 - 2.47217 =
    // 5.02789 and shift_down = d + sub_drop(7pt) = 2.50006 + 0.5 = 3.00006.
    // \showbox: \hbox(8.04175+3.00005)x24.00992, scripts vbox shifted 3.00005.
    let list = fixtures::sum_limits();
    let b = layout(&list, Style::TEXT, &cm());
    assert_eq!(dims(&b), "(8.04175+3.00005)x24.00992");
    let r = runs(&list, Style::TEXT);
    assert_eq!(glyph(&r, '\u{2211}').gid, 0x50);
    // Scripts start after the operator's advance; n and i share an x.
    assert_eq!(f5(glyph(&r, 'n').x), "10.55559");
    assert_eq!(f5(glyph(&r, 'i').x), "10.55559");
}

#[test]
fn integral_scripts_use_italic_correction() {
    // \int has \nolimits: in display the large ∫ (gid 0x5A) has an italic
    // correction of 4.44444pt, and Rule 18 places the superscript δ further
    // right than the subscript. \showbox: \hbox(15.65013+9.11122)x14.48615,
    // "1" box shifted 4.44444 inside the scripts vbox.
    let list = fixtures::int_0_1();
    let b = layout(&list, Style::DISPLAY, &cm());
    assert_eq!(dims(&b), "(15.65014+9.11121)x14.48615");
    let r = runs(&list, Style::DISPLAY);
    let one = glyph(&r, '1');
    let zero = glyph(&r, '0');
    assert_eq!(f5(one.x - zero.x), "4.44444");
}

#[test]
fn accent_hat_uses_skew_and_x_height() {
    // Rule 12: the accent (cmr10 "^", 5.00002 wide, 6.94444 high) is centred
    // over x (5.71527 wide) and shifted right by x's skew kern with the skew
    // character (0.27779pt): 0.27779 + (5.71527 - 5.00002)/2 = 0.63541. It is
    // lowered by δ = min(h(x), x_height) = 4.30554 so its baseline coincides
    // with x's; the box keeps the accent's full height.
    // \showbox: \hbox(6.94444+0.0)x5.71527, "^" shifted 0.63542, \kern-4.30554.
    let list = fixtures::hat_x();
    let b = layout(&list, Style::TEXT, &cm());
    assert_eq!(dims(&b), "(6.94444+0.00000)x5.71527");
    let r = runs(&list, Style::TEXT);
    let hat = glyph(&r, '^');
    let x = glyph(&r, 'x');
    assert_eq!(f5(hat.x), "0.63541");
    assert_eq!(f5(hat.baseline_y), f5(x.baseline_y));
}

#[test]
fn spacing_classes_follow_the_texbook_table() {
    // mu = quad/18 with TeX's truncation: 36408sp = 0.55554pt at 10pt.
    // Ord–Bin–Ord gets \medmuskip (4mu = 2.22217pt) on both sides of "+";
    // Ord–Rel–Ord gets \thickmuskip (5mu = 2.77771pt); Ord–Open and
    // Close–Ord get nothing; Punct–Ord gets \thinmuskip (3mu = 1.66663pt).
    // \showbox widths: a+b 21.79968, a=b 22.91077, f(x) 19.46533, a,b 14.02196.
    let m = cm();
    let width = |l: &MathList| f5(layout(l, Style::TEXT, &m).width);
    assert_eq!(width(&fixtures::a_plus_b()), "21.79968");
    assert_eq!(width(&fixtures::a_eq_b()), "22.91077");
    assert_eq!(width(&fixtures::f_of_x()), "19.46533");
    assert_eq!(width(&fixtures::a_comma_b()), "14.02196");
    let r = runs(&fixtures::a_plus_b(), Style::TEXT);
    // a is 5.28589 wide, then 2.22217 of glue, then "+".
    assert_eq!(f5(glyph(&r, '+').x), f5(5.28589 + 2.22217));
    // f(x): f's italic correction (1.0764pt) is a kern before "(".
    let r = runs(&fixtures::f_of_x(), Style::TEXT);
    assert_eq!(f5(glyph(&r, '(').x), f5(4.89586 + 1.0764));
    // In script style the medium and thick spaces vanish.
    let script = layout(&fixtures::a_plus_b(), Style::SCRIPT, &m);
    let glue: Vec<f64> = match &script.kind {
        BoxKind::HBox(children) => children
            .iter()
            .filter(|c| matches!(c.content.kind, BoxKind::Glue { .. }))
            .map(|c| c.content.width)
            .collect(),
        _ => unreachable!(),
    };
    assert!(glue.is_empty());
}

#[test]
fn bin_atoms_become_ord_at_list_edges() {
    // Rule 5: a leading or trailing "-" is Ord, so "-a" has no medium space
    // and "a-" likewise; "a--b" makes the second "-" Ord (after a Bin).
    let m = cm();
    let minus_a = layout(&MathList::symbols("-a"), Style::TEXT, &m);
    let a = m_width(&m, 'a');
    let minus = m_width(&m, '-');
    assert_eq!(f5(minus_a.width), f5(a + minus));
    let a_minus = layout(&MathList::symbols("a-"), Style::TEXT, &m);
    assert_eq!(f5(a_minus.width), f5(a + minus));
    let double = layout(&MathList::symbols("a--b"), Style::TEXT, &m);
    let b = m_width(&m, 'b');
    let mu4 = 145632.0 / 65536.0;
    assert_eq!(f5(double.width), f5(a + mu4 + minus + minus + mu4 + b));
}

fn m_width(m: &CmMathMetrics, ch: char) -> f64 {
    use flashtex_math_layout::SizeClass;
    m.glyph(ch, SizeClass::Text).unwrap().width
}

#[test]
fn layout_is_deterministic_and_pure() {
    let m = cm();
    for (_, list) in fixtures::all() {
        for style in [
            Style::DISPLAY,
            Style::TEXT,
            Style::SCRIPT,
            Style::SCRIPT_SCRIPT,
        ] {
            let a = layout_with_report(&list, style, &m);
            let b = layout_with_report(&list, style, &m);
            assert_eq!(a, b);
            assert_eq!(
                positioned_runs(&a.root, (72.0, 700.0)),
                positioned_runs(&b.root, (72.0, 700.0))
            );
        }
    }
}

#[test]
fn positioned_runs_translate_by_origin() {
    let list = fixtures::x_squared();
    let b = layout(&list, Style::TEXT, &cm());
    let at_origin = positioned_runs(&b, (0.0, 0.0));
    let moved = positioned_runs(&b, (10.0, 20.0));
    for (g0, g1) in at_origin.glyphs.iter().zip(&moved.glyphs) {
        assert_eq!(f5(g1.x - g0.x), "10.00000");
        assert_eq!(f5(g1.baseline_y - g0.baseline_y), "20.00000");
    }
    // The origin is the top-left corner: x's baseline is `height` below it.
    assert_eq!(f5(at_origin.glyphs[0].baseline_y), f5(b.height));
}

#[test]
fn times_approximation_lays_out_everything_with_reported_limitations() {
    // The Times adapter has no size chains; the layout still completes and
    // reports what it could not size. This is the fallback path when no
    // Computer Modern renderer is available.
    let m = TimesApproxMetrics::new(10.0);
    for (name, list) in fixtures::all() {
        let out = layout_with_report(&list, Style::DISPLAY, &m);
        assert!(out.root.width > 0.0, "{name} has zero width");
        for l in &out.limitations {
            assert!(
                matches!(
                    l,
                    Limitation::DelimiterTooSmall { .. } | Limitation::RadicalTooSmall { .. }
                ),
                "{name}: unexpected limitation {l:?}"
            );
        }
    }
    // Fraction bar geometry is explicit even with approximate glyphs.
    let r = positioned_runs(&layout(&fixtures::frac_a_b(), Style::TEXT, &m), (0.0, 0.0));
    assert_eq!(r.rules.len(), 1);
    assert_eq!(f5(r.rules[0].h), "0.39998");
}
